use std::path::PathBuf;
use structopt::StructOpt;

pub const CONFIG_SYS_PATH: &str = "/etc/vehicle-nav/config.toml";

#[derive(Debug, Clone, PartialEq, StructOpt)]
pub struct Opts {
    /// Write the default configuration file to path and exit
    #[structopt(long, name = "path")]
    pub write_default_config: Option<PathBuf>,

    /// Configuration file path
    #[structopt(long, short = "c", default_value = CONFIG_SYS_PATH)]
    pub config: PathBuf,
}
