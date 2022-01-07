use std::error::Error;
use clap::StructOpt;
use cli_args::{Cli, Commands};
use crate::app_config::AppInitializer;

mod cli_args;
mod app_config;

fn main() -> Result<(), Box<dyn Error>> {
    AppInitializer::init()?;
    
    let args = Cli::parse();

    match &args.command {
        Commands::Init => {
            println!("INIT CHOSEN");

        },
        Commands::Add { file } => {
            println!("FILE CHOSEN");
        }
    }

    Ok(())
}

// TODO: Improve error handling for good error UX