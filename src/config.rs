use std::path::{Path, PathBuf};
use clap::Parser;
use regex::Regex;
use std::process::exit;
use crate::print_error;

#[derive(PartialEq, Debug)]
pub enum FindType {
    File,
    Dir,
    Symlink,
}

#[derive(PartialEq, Debug)]
pub enum SizeType {
    Eq(u64),
    Gte(u64),
    Le(u64),
}

#[derive(Debug)]
enum SizeTypeParseError {
    NonDecodeableSize,
}

impl SizeType {
    fn parse(s: &str) -> Result<Self, SizeTypeParseError> {
        let re = Regex::new(r"^([+-]?)([0-9]+)([KMG]?)$").unwrap();
        if let Some(caps) = re.captures(s) {
            let multiplier = match caps.get(3).unwrap().as_str() {
                "K" => 1024,
                "M" => 1024 * 1024,
                "G" => 1024 * 1024 * 1024,
                "" => 1,
                _ => return Err(SizeTypeParseError::NonDecodeableSize),
            };
            let value_in_bytes = caps.get(2).unwrap().as_str().parse().unwrap();
            match caps.get(1).unwrap().as_str() {
                "+" => Ok(SizeType::Gte(value_in_bytes * multiplier)),
                "-" => Ok(SizeType::Le(value_in_bytes * multiplier)),
                _ => Ok(SizeType::Eq(value_in_bytes)),
            }
        } else {
            Err(SizeTypeParseError::NonDecodeableSize)
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub dir: PathBuf,
    pub find_type: FindType,
    pub size_in_bytes: Option<SizeType>,
    pub regex: Option<Regex>,
    pub depth: Option<u16>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            dir: Path::new(".").to_owned(),
            find_type: FindType::File,
            size_in_bytes: None,
            regex: None,
            depth: None,
        }
    }
}

#[derive(Debug)]
enum NameMatcherError {
    MoreThanOneMatcher,
}

fn create_name_matcher(
    regex: Option<String>,
    name: Option<String>,
    iname: Option<String>,
) -> Result<Regex, NameMatcherError> {
    match (regex, name, iname) {
        (Some(regex), None, None) => Ok(Regex::new(&regex).unwrap()),
        (None, Some(name), None) => {
            let fixed_name = name.replace("*", "(.*)");
            Ok(Regex::new(&format!("^{}$", fixed_name)).unwrap())
        }
        (None, None, Some(iname)) => {
            let fixed_name = iname.replace("*", "(.*)");
            Ok(Regex::new(&format!("(?i)^{}$", fixed_name)).unwrap())
        }
        (None, None, None) => unreachable!(),
        _ => Err(NameMatcherError::MoreThanOneMatcher),
    }
}

impl Config {
    pub fn parse() -> Config {
        let raw_config = RawConfig::parse();
        let mut config = Config::default();
        config.dir = Path::new(&raw_config.dir).to_owned();

        if raw_config.find_type == "d" {
            config.find_type = FindType::Dir;
        } else if raw_config.find_type == "s" {
            config.find_type = FindType::Symlink;
        } else if raw_config.find_type != "f" {
            print_error!("unknown type: {}", raw_config.find_type);
            exit(1)
        }

        if let Some(size) = raw_config.size {
            let st = SizeType::parse(&size);
            if st.is_err() {
                print_error!("unknown size type: {}", size);
                exit(1);
            }
            config.size_in_bytes = Some(st.unwrap());
        }

        if raw_config.regex.is_some() ||
            raw_config.name.is_some() ||
            raw_config.iname.is_some() {
            let matcher_result =
                create_name_matcher(raw_config.regex, raw_config.name, raw_config.iname);
            if matcher_result.is_err() {
                print_error!("name matcher can't be decoded: {:?}", matcher_result.unwrap_err());
                exit(1);
            }
            config.regex = Some(matcher_result.unwrap());
        }
        if let Some(depth) = raw_config.depth {
            if depth == 0 {
                print_error!("depth should be >0");
                exit(1);
            }
            config.depth = Some(depth);
        }

        config
    }
}

#[derive(Debug, Parser)]
struct RawConfig {
    dir: String,
    #[arg(long = "type", default_value = "f")]
    find_type: String,
    #[arg(long, allow_hyphen_values = true)]
    size: Option<String>,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    iname: Option<String>,
    #[arg(long)]
    regex: Option<String>,
    #[arg(long)]
    depth: Option<u16>,
}
