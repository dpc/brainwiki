use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt, Clone)]
pub enum Command {
    #[structopt(name = "passwd")]
    /// Set password
    Password,
}

#[derive(Debug, StructOpt, Clone)]
#[structopt(
    name = "brainweb", about = "A personal braindump wiki"
)]
pub struct Opts {
    #[structopt(
        long = "data-dir",
        short = "d",
        parse(from_os_str),
        default_value = "./data"
    )]
    pub data_dir: PathBuf,
    #[structopt(
        long = "theme-dir",
        parse(from_os_str),
        default_value = "./theme"
    )]
    pub theme_dir: PathBuf,

    #[structopt(subcommand)]
    pub command: Option<Command>,
    /// Run locally - no password needed for editing
    pub local: bool,
    //    #[structopt(flatten)]
    //    verbosity: Verbosity,
}
