use clap::StructOpt;
use cli_args::{Cli, Commands};

mod cli_args;

fn main() {
    let args = Cli::parse();

    match &args.command {
        Commands::Init => {
            println!("INIT CHOSEN");
        },
        Commands::Add { file } => {
            println!("FILE CHOSEN");
        }
    }
}
