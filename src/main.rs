use app_init::AppInitializer;
use cli_args::{AppArgs, Command};
use commands::{add_command::AddCommand, init_command::InitCommand, list_command::ListCommand, rm_command::RmCommand};
use config::{
    app_config::AppConfigManager, locations::LocationsProvider, projects::ProjectsRetriever,
};
use std::{env, error::Error, path::Path};
use structopt::StructOpt;

mod app_init;
mod cli_args;
mod commands;
mod config;
mod error;
mod fs_utils;
mod project_init;
mod git_ignore;

fn main() -> Result<(), Box<dyn Error>> {
    let args = AppArgs::from_args();

    let locations_provider = match args.config_path.as_str() {
        "default" => LocationsProvider::default(),
        other => LocationsProvider::new(Path::new(other).to_path_buf()),
    };

    AppInitializer {
        locations_provider: &locations_provider,
    }
    .init()?;

    let app_config_manager = AppConfigManager {
        config_file_path: locations_provider.get_config_file_path(),
    };
    let app_config = app_config_manager.get_config()?;

    match args.command {
        Command::Init => {
            let retriever = ProjectsRetriever::new(app_config, &locations_provider);
            let cwd = env::current_dir()?;

            let command = InitCommand {
                projects_retriever: &retriever,
                app_config_manager: &app_config_manager,
                locations_provider: &locations_provider,
            };
            command.init(&cwd)?;
        }
        Command::Add { file, git_ignore } => {
            let cwd = env::current_dir()?;
            let command = AddCommand::new(&locations_provider);
            command.add_file(file, &cwd, git_ignore)?;
        }
        Command::List(options) => {
            let projects_retriever =
                ProjectsRetriever::new(app_config, &locations_provider);
            let command = ListCommand::new(&projects_retriever);
            command.list(options.only_associated, options.only_unassociated)?;
        }
        Command::Rm { file, delete_file } => {
            let cwd = env::current_dir()?;
            let projects_retriever =
                ProjectsRetriever::new(app_config, &locations_provider);
            let command = RmCommand::new(&locations_provider, &projects_retriever);
            command.remove_file(file, &cwd, delete_file)?;
        },
    }

    Ok(())
}

// TODO: Improve error handling for good error UX
