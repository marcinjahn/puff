use std::error::Error;
use clap::StructOpt;
use cli_args::{Cli, Commands};
use commands::{init_command, add_command};

mod cli_args;
mod app_init;
mod config;
mod project_init;
mod error;
mod commands;
mod fs_utils;

fn main() -> Result<(), Box<dyn Error>> {
    app_init::init()?;
    
    let args = Cli::parse();

    match args.command {
        Commands::Init => {
            init_command::init()?;
        },
        Commands::Add { file } => {
            add_command::add_file(file)?;
        }
    }

    Ok(())
}

// TODO: Improve error handling for good error UX