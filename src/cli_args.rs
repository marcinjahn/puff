use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct AppArgs {
    #[structopt(default_value = "default", env, hidden = true)]
    pub config_path: String,

    #[structopt(subcommand)]
    pub command: Command,
}

#[derive(StructOpt)]
#[structopt(
    name = "conman",
    about = "A configuration manager that keeps private configuration files from various projects in a central directory so that they can be easily synced between different dev machines."
)]
pub enum Command {
    /// Adds a new file to be tracked by conman in this project.
    /// If the file does not exist, it will be created.
    Add {
        file: PathBuf, // TODO: Vec<PathBuf>
    },

    /// Initializes the project.
    Init,
}
