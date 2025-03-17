use std::path::PathBuf;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct StartParam {
    #[structopt(
        short = "c",
        long = "conifg",
        parse(from_os_str),
        default_value = "config.toml"
    )]
    pub config_file: PathBuf,
}
