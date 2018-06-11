use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "brainweb", about = "A personal braindump wiki")]
pub struct Opts {
    #[structopt(long = "dir", parse(from_os_str))]
    dir: PathBuf,
}

pub fn from_args() -> Opts {
    Opts::from_args()
}

