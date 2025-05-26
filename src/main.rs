#![feature(let_chains)]

mod config;

use crate::config::{FindType, SizeType};
use config::Config;
use std::fs;
use std::path::Path;
use std::process::exit;

#[macro_export]
macro_rules! print_error {
    ($($arg:tt)*) => {
        eprintln!("find: {}", format!($($arg)*))
    };
}

#[derive(Eq, PartialEq, Debug)]
enum PathType {
    Directory,
    File,
    Symlink,
    Unknown,
}

fn get_type(path: &Path) -> PathType {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_dir() => PathType::Directory,
        Ok(meta) if meta.file_type().is_symlink() => PathType::Symlink,
        Ok(meta) if meta.file_type().is_file() => PathType::File,
        _ => PathType::Unknown,
    }
}

struct Find {
    config: Config,
    depth: u16,
}

impl Find {
    fn new(config: Config) -> Self {
        Self { config, depth: 0 }
    }

    fn match_name(&self, path: &Path) -> bool {
        match &self.config.regex {
            None => true,
            Some(matcher) => {
                let maybe_file_name =
                    path.file_name().and_then(|os_str| os_str.to_str());
                match maybe_file_name {
                    None => false,
                    Some(file_name) => matcher.is_match(file_name),
                }
            }
        }
    }

    fn match_size(&self, path: &Path) -> bool {
        let size = match path.metadata() {
            Ok(m) => m.len(),
            Err(_) => return false,
        };
        match &self.config.size_in_bytes {
            None => true,
            Some(SizeType::Gte(s)) => size >= *s,
            Some(SizeType::Eq(s)) => size == *s,
            Some(SizeType::Le(s)) => size <= *s,
        }
    }


    fn find_type_is_dir(&self) -> bool {
        self.config.find_type == FindType::Dir
    }

    fn file_matches(&self, path: &Path) -> bool {
        self.config.find_type == FindType::File
            && self.match_name(&path)
            && self.match_size(&path)
    }

    fn symlink_matches(&self, path: &Path) -> bool {
        self.config.find_type == FindType::Symlink
            && self.match_name(&path)
    }

    pub fn run(&mut self, path: &Path) {
        match get_type(&path) {
            PathType::File if self.file_matches(path) => println!("{}", path.display()),
            PathType::Symlink if self.symlink_matches(path) => println!("{}", path.display()),
            PathType::Directory => {
                if self.find_type_is_dir() {
                    println!("{}", path.display());
                }
                self.inspect_dir(path);
            }
            _ => {}
        }
    }

    fn inspect_dir(&mut self, path: &Path) {
        if let Some(depth) = self.config.depth && self.depth == depth {
            return;
        }

        self.depth += 1;

        match fs::read_dir(path) {
            Err(e) => print_error!("{}", e),
            Ok(entries) => {
                for entry in entries {
                    if let Ok(entry) = entry {
                        self.run(&entry.path());
                    } else {
                        print_error!("{}: ", entry.err().unwrap());
                    }
                }
            }
        }

        self.depth -= 1;
    }
}

fn main() {
    let config = Config::parse();

    if !config.dir.exists() {
        print_error!("{}: no such file or directory", config.dir.display());
        exit(1);
    }

    let root_path = config.dir.clone();
    let mut find = Find::new(config);

    find.run(&root_path);
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix;
    use crate::{get_type, Find, PathType};
    use std::path::PathBuf;
    use regex::Regex;
    use crate::config::Config;

    #[test]
    fn test_match_name() {
        let mut config = Config::default();
        config.regex = Some(Regex::new("^(.*).rs$").unwrap());
        let find = Find::new(config);

        assert_eq!(find.match_name(&PathBuf::from("./src/main.rs")), true);
        assert_eq!(find.match_name(&PathBuf::from("./src/config.rs")), true);
    }

    #[test]
    fn test_get_type() {
        assert_eq!(get_type(&PathBuf::from("/foo")), PathType::Unknown);
        assert_eq!(get_type(&PathBuf::from("./src/main.rs")), PathType::File);
        assert_eq!(get_type(&PathBuf::from("./src")), PathType::Directory);

        unix::fs::symlink(&PathBuf::from("./src/main.rs"),
                          &PathBuf::from("./main.rs.sl")).unwrap();

        assert_eq!(get_type(&PathBuf::from("./main.rs.sl")), PathType::Symlink);

        fs::remove_file(&PathBuf::from("./main.rs.sl")).unwrap()
    }
}
