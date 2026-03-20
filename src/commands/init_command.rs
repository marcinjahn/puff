use crate::{
    config::{
        app_config::AppConfigManager, locations::LocationsProvider, projects::ProjectsRetriever,
    },
    io_utils::{prompt_input, prompt_select},
    project_init::existing::ExistingProjectInitializer,
};
use anyhow::{Result, bail};
use std::{fs, path::Path};

pub struct InitCommand<'a> {
    pub projects_retriever: &'a ProjectsRetriever<'a>,
    pub app_config_manager: &'a AppConfigManager,
    pub locations_provider: &'a LocationsProvider,
}

impl<'a> InitCommand<'a> {
    pub fn init(
        &self,
        cwd: &Path,
        name: Option<String>,
        associate: Option<String>,
    ) -> Result<()> {
        if self.projects_retriever.is_associated(cwd)? {
            bail!("This directory is already initialized with puff.");
        }

        if let Some(project_name) = associate {
            let unassociated = self.projects_retriever.get_unassociated_projects()?;
            if !unassociated.contains(&project_name) {
                bail!("Project '{}' is not an unassociated project.", project_name);
            }
            self.associate_project(&project_name, cwd)?;
        } else if let Some(project_name) = name {
            self.init_fresh_project(&project_name, cwd)?;
        } else {
            let unassociated = self.projects_retriever.get_unassociated_projects()?;
            if !unassociated.is_empty() {
                self.handle_with_unassociated(unassociated, cwd)?;
            } else {
                let project_name = self.prompt_project_name(cwd)?;
                self.init_fresh_project(&project_name, cwd)?;
            }
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

    fn associate_project(&self, name: &str, cwd: &Path) -> Result<()> {
        let existing_initializer = ExistingProjectInitializer::new(self.app_config_manager);
        existing_initializer.init_project(name, cwd, &self.locations_provider.get_managed_dir(name))
    }

    fn handle_with_unassociated(&self, unassociated: Vec<String>, cwd: &Path) -> Result<()> {
        println!("Some projects in puff are not yet associated with a path on this machine.");
        let choice = self.ask_about_unassociated(&unassociated)?;
        match choice {
            UserChoice::Fresh => {
                let name = self.prompt_project_name(cwd)?;
                self.init_fresh_project(&name, cwd)?;
            }
            UserChoice::Existing(name) => {
                self.associate_project(name, cwd)?;
            }
        }

        Ok(())
    }

    fn prompt_project_name(&self, cwd: &Path) -> Result<String> {
        let default = cwd
            .file_name()
            .and_then(|s| s.to_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_owned());

        prompt_input("Project name", default)
    }

    fn ask_about_unassociated(&self, unassociated: &'a [String]) -> Result<UserChoice<'a>> {
        let mut items: Vec<String> = vec!["Create a new project".to_owned()];
        for project in unassociated {
            items.push(format!("Associate with '{}'", project));
        }

        let selection = prompt_select(
            "Associate with an existing project or create new",
            &items,
        )?;

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
