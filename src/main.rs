use std::error::Error;
use clap::StructOpt;
use cli_args::{Cli, Commands};

mod cli_args;
mod app_init;
mod config;
mod project_init;
mod error;

fn main() -> Result<(), Box<dyn Error>> {
    app_init::init()?;
    
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