use crate::{
    config::{
        app_config::AppConfigManager, locations::LocationsProvider, projects::ProjectsRetriever,
    },
    project_init::existing::ExistingProjectInitializer,
};
use anyhow::{Result, bail};
use dialoguer::{Input, Select};
use std::{fs, path::Path};

pub struct InitCommand<'a> {
    pub projects_retriever: &'a ProjectsRetriever<'a>,
    pub app_config_manager: &'a AppConfigManager,
    pub locations_provider: &'a LocationsProvider,
}

impl<'a> InitCommand<'a> {
    pub fn init(&self, cwd: &Path) -> Result<()> {
        if self.projects_retriever.is_associated(cwd)? {
            bail!("This directory is already initialized with puff.");
        }

        let unassociated = self.projects_retriever.get_unassociated_projects()?;
        if !unassociated.is_empty() {
            self.handle_with_unassociated(unassociated, cwd)?;
        } else {
            let name = self.get_fresh_project_name(cwd)?;
            self.init_fresh_project(&name, cwd)?;
        }

        println!("Project initialized.");

        Ok(())
    }

    fn init_fresh_project(&self, name: &str, user_dir: &Path) -> Result<()> {
        let managed_dir = self.locations_provider.get_managed_dir(name);
        if managed_dir.exists() {
            bail!("A project with this name already exists in puff's registry.");
        }

        fs::create_dir_all(managed_dir)?;
        self.app_config_manager.add_project(name, user_dir)?;

        Ok(())
    }

    fn handle_with_unassociated(&self, unassociated: Vec<String>, cwd: &Path) -> Result<()> {
        println!("Some projects in puff are not yet associated with a path on this machine.");
        println!("Associate one with the current directory, or create a new project.");
        let choice = self.ask_about_unassociated(&unassociated)?;
        match choice {
            UserChoice::Fresh => {
                let name = self.get_fresh_project_name(cwd)?;
                self.init_fresh_project(&name, cwd)?;
            }
            UserChoice::Existing(name) => {
                let existing_initializer = ExistingProjectInitializer::new(self.app_config_manager);
                existing_initializer.init_project(
                    name,
                    cwd,
                    &self.locations_provider.get_managed_dir(name),
                )?
            }
        }

        Ok(())
    }

    fn get_fresh_project_name(&self, cwd: &Path) -> Result<String> {
        let mut input = Input::<String>::new().with_prompt("Project name");

        if let Some(name) = cwd.file_name().and_then(|s| s.to_str()).filter(|s| !s.is_empty()) {
            input = input.default(name.to_owned());
        }

        input
            .validate_with(|s: &String| {
                if s.trim().is_empty() {
                    Err("Name cannot be empty")
                } else {
                    Ok(())
                }
            })
            .interact_text()
            .map_err(Into::into)
    }

    fn ask_about_unassociated(&self, unassociated: &'a [String]) -> Result<UserChoice<'a>> {
        let mut items: Vec<String> = vec!["Create a new project".to_owned()];
        for project in unassociated {
            items.push(format!("Associate with '{}'", project));
        }

        let selection = Select::new()
            .with_prompt("Associate with an existing project or create new")
            .items(&items)
            .default(0)
            .interact()?;

        if selection == 0 {
            Ok(UserChoice::Fresh)
        } else {
            Ok(UserChoice::Existing(&unassociated[selection - 1]))
        }
    }
}

enum UserChoice<'a> {
    Fresh,
    Existing(&'a str),
}
