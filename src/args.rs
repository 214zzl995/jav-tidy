use std::path::PathBuf;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct StartParam {
    #[structopt(
        short = "c",
        long = "config",
        parse(from_os_str),
        default_value = "config.toml"
    )]
    pub config_file: PathBuf,

    #[cfg(unix)]
    #[cfg(not(debug_assertions))]
    #[structopt(
        short = "l",
        long = "log",
        parse(from_os_str),
        default_value = "/var/lib/javtidy/log"
    )]
    pub log_location: PathBuf,

    #[cfg(windows)]
    #[cfg(not(debug_assertions))]
    #[structopt(short = "l", long = "log", parse(from_os_str), default_value = "./log")]
    pub log_location: PathBuf,

    #[cfg(debug_assertions)]
    #[structopt(short = "l", long = "log", parse(from_os_str), default_value = "./log")]
    pub log_location: PathBuf,

    #[cfg(debug_assertions)]
    #[structopt(
        short = "t",
        long = "template",
        parse(from_os_str),
        default_value = "./template"
    )]
    pub template_location: PathBuf,

    #[cfg(not(debug_assertions))]
    #[structopt(
        short = "t",
        long = "template",
        parse(from_os_str),
        default_value = "/var/lib/javtidy/template"
    )]
    pub template_location: PathBuf,
}
