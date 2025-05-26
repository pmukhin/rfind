# rfind

Rust implementation of Unix find command.

## How to run

### Matching by name

#### Find all files by name case-insensitively

`rfind <where> --iname <matcher>`

#### Find all files by name case-sensitively

`rfind <where> --name <matcher>`

#### Find all files by name using a regular expression

`rfind <where> --regex <matcher>`

### Matching by file size

#### Find all files with size > 10MB

`rfind <where> --size +10M`

#### Find all files with size > 10KB

`rfind <where> --size +10K`

#### Find all files with size > 10GB

`rfind <where> --size +10G`

#### Find all files with size > 10B

`rfind <where> --size +10`

#### Find all files with size = 10GB

`rfind <where> --size 10G`

#### Find all files with size < 10GB

`rfind <where> --size -10G`

### Match directories or symlinks instead of files

#### Match directories

`rfind <where> --type d`

#### Match symlinks

`rfind <where> --type s`

#### Match files

`rfind <where> --type f`

### Other

#### Allow specific depth

`rfind <where> --depth 12`

### To know

#### Matchers are combinable

`rfind <where> --type f --iname *.log --size +10G --depth 12`
