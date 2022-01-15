use app_init::AppInitializer;
use clap::StructOpt;
use cli_args::{Cli, Commands};
use commands::{add_command::AddCommand, init_command::InitCommand};
use config::{
    app_config::AppConfigManager, locations::LocationsProvider, projects::ProjectsRetriever,
};
use std::{env, error::Error};

mod app_init;
mod cli_args;
mod commands;
mod config;
mod error;
mod fs_utils;
mod project_init;
mod test_utils;

fn main() -> Result<(), Box<dyn Error>> {
    let locations_provider = LocationsProvider::default();
    let app_config_manager = AppConfigManager {
        config_file_path: locations_provider.get_config_file_path(),
    };
    let app_config = app_config_manager.get_config()?;

    AppInitializer {
        locations_provider: &locations_provider,
    }
    .init()?;

    let args = Cli::parse();

    match args.command {
        Commands::Init => {
            let retriever = ProjectsRetriever::new(app_config, &locations_provider);
            let cwd = env::current_dir()?;

            let command = InitCommand {
                projects_retriever: &retriever,
                app_config_manager: &app_config_manager,
                locations_provider: &locations_provider,
            };
            command.init(&cwd)?;
        }
        Commands::Add { file } => {
            let cwd = env::current_dir()?;
            let command = AddCommand::new(&locations_provider);
            command.add_file(file, &cwd)?;
        }
    }

    Ok(())
}

// TODO: Improve error handling for good error UX
