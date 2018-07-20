use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt, Clone)]
enum Command {
    #[structopt(name = "gen")]
    /// Generate
    Generate,
}

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

    #[structopt(subcommand)]
    command: Option<Command>,
    /// Run locally - no password needed for editing
    local: bool,
    //    #[structopt(flatten)]
    //    verbosity: Verbosity,
}
