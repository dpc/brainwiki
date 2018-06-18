use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt, Clone)]
#[structopt(
    name = "brainweb", about = "A personal braindump wiki"
)]
pub struct Opts {
    #[structopt(
        long = "datadir",
        short = "d",
        parse(from_os_str),
        default_value = "./data"
    )]
    pub data_dir: PathBuf,
    #[structopt(
        long = "theme_dir",
        parse(from_os_str),
        default_value = "./theme"
    )]
    pub theme_dir: PathBuf,
}

pub fn from_args() -> Opts {
    Opts::from_args()
}
