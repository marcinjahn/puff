use std::error::Error;
use clap::StructOpt;
use cli_args::{Cli, Commands};
use commands::init_command::init;

mod cli_args;
mod app_init;
mod config;
mod project_init;
mod error;
mod commands;

fn main() -> Result<(), Box<dyn Error>> {
    app_init::init()?;
    
    let args = Cli::parse();

    match &args.command {
        Commands::Init => {
            init()?;
        },
        Commands::Add { file: _ } => {
            println!("FILE CHOSEN");
        }
    }

    Ok(())
}

// TODO: Improve error handling for good error UX