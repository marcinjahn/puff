use clap::{AppSettings, Parser, Subcommand};
use std::{path::PathBuf};

#[derive(Subcommand)]
pub enum Commands {
    /// Adds a new file to be tracked by conman in this project.
    /// If the file does not exist, it will be created.
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Add {
        file: PathBuf, // TODO: Vec<PathBuf>
    },

    /// Initializes the project.
    Init
}

#[derive(Parser)]
#[clap(name = "conman")]
#[clap(about = "A configuration manager that keeps private configuration files from various projects in a central directory so that they can be easily synced between different dev machines.")]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands
}