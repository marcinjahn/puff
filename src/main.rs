use anyhow::Result;
use app_init::AppInitializer;
use cli_args::{AppArgs, Command};
use clap::Parser;
use commands::{
    add_command::AddCommand, file_forget_command::ForgetCommand, init_command::InitCommand,
    list_command::ListCommand, project_forget_command::ProjectForgetCommand,
};
use config::{
    app_config::AppConfigManager, locations::LocationsProvider, projects::ProjectsRetriever,
};
use std::{env, path::Path};

mod app_init;
mod cli_args;
mod commands;
mod config;
mod fs_utils;
mod git_ignore;
mod io_utils;
mod project_init;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = AppArgs::parse();

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
        Command::Add { files, git_ignore } => {
            let cwd = env::current_dir()?;
            let command = AddCommand::new(&locations_provider);
            let mut had_error = false;
            for file in files {
                if let Err(e) = command.add_file(file, &cwd, git_ignore) {
                    eprintln!("Error: {e}");
                    had_error = true;
                }
            }
            if had_error {
                std::process::exit(1);
            }
        }
        Command::List(options) => {
            let projects_retriever = ProjectsRetriever::new(app_config, &locations_provider);
            let command = ListCommand::new(&projects_retriever);
            command.list(options.only_associated, options.only_unassociated)?;
        }
        Command::Forget { files, delete_file } => {
            let cwd = env::current_dir()?;
            let command = ForgetCommand::new(&locations_provider);
            let mut had_error = false;
            for file in files {
                if let Err(e) = command.forget_file(file, &cwd, delete_file) {
                    eprintln!("Error: {e}");
                    had_error = true;
                }
            }
            if had_error {
                std::process::exit(1);
            }
        }
        Command::Project { subcommand } => match subcommand {
            cli_args::ProjectSubcommand::Forget(details) => {
                let projects_retriever = ProjectsRetriever::new(app_config, &locations_provider);
                let command = ProjectForgetCommand::new(&projects_retriever, &app_config_manager);
                command.forget_project(
                    details.project_name,
                    details.delete_files,
                    details.skip_confirmation,
                )?;
            }
        },
    }

    Ok(())
}
