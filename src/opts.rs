use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt, Clone)]
#[structopt(name = "brainweb", about = "A personal braindump wiki")]
pub struct Opts {
    #[structopt(long = "datadir", short = "d", parse(from_os_str))]
    pub data_dir: PathBuf,
}

pub fn from_args() -> Opts {
    Opts::from_args()
}
