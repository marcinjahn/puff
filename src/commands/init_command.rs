use crate::{
    config::{
        app_config::AppConfigManager, locations::LocationsProvider, projects::ProjectsRetriever,
    },
    error::AppError,
    project_init::existing::{ExistingProjectInitializer},
};
use std::{error::Error, fs, path::Path};

pub struct InitCommand<'a> {
    pub projects_retriever: &'a ProjectsRetriever<'a>,
    pub app_config_manager: &'a AppConfigManager,
    pub locations_provider: &'a LocationsProvider,
}

impl<'a> InitCommand<'a> {
    pub fn init(&self, cwd: &Path) -> Result<(), Box<dyn Error>> {
        if self.projects_retriever.is_associated(&cwd)? {
            return Err(Box::new(AppError(
                "This project is already configured in conman".into(),
            )));
        }

        let unassociated = self.projects_retriever.get_unassociated_projects()?;
        if !unassociated.is_empty() {
            self.handle_with_unassociated(unassociated, &cwd)?;
        } else {
            let name = self.get_fresh_project_name(&cwd)?;
            self.init_fresh_project(&name, cwd)?;
        }

        println!("Project has been set up with conman");

        Ok(())
    }

    fn init_fresh_project(&self, name: &str, user_dir: &Path) -> Result<(), Box<dyn Error>> {
        let managed_dir = self.locations_provider.get_managed_dir(name);
        if managed_dir.exists() {
            return Err(Box::new(AppError(
                "The project folder already exists in conman's configs".into(),
            )));
        }

        fs::create_dir_all(managed_dir)?;
        self.app_config_manager.add(name, user_dir)?;

        Ok(())
    }

    fn handle_with_unassociated(
        &self,
        unassociated: Vec<String>,
        cwd: &Path,
    ) -> Result<(), Box<dyn Error>> {
        println!("conman has a few projects that are still not associated with any path on your machine. Do you want");
        println!(
            "to associate one of them with the current path, or do you want to set up a fresh project?"
        );
        let choice = self.ask_about_unassociated(&unassociated)?;
        match choice {
            UserChoice::Fresh => {
                let name = self.get_fresh_project_name(cwd)?;
                self.init_fresh_project(&name, cwd)?;
            }
            UserChoice::Existing(name) => {
                let existing_initializer = ExistingProjectInitializer {
                    app_configuration_manager: self.app_config_manager,
                };
                existing_initializer.init_project(
                    name,
                    cwd,
                    &self.locations_provider.get_managed_dir(name),
                )?
            }
        }

        Ok(())
    }

    fn get_fresh_project_name(&self, cwd: &Path) -> Result<String, Box<dyn Error>> {
        let mut proposed_name = String::new();
        if let Some(osstr) = cwd.file_name() {
            if let Some(osstr) = osstr.to_str() {
                proposed_name = osstr.to_owned();
            }
        }

        if !proposed_name.is_empty() {
            println!("Provide a name for this new project ({}): ", proposed_name);
        } else {
            println!("Provide a name for this new project: ");
        }

        let mut user_name = String::new();
        std::io::stdin().read_line(&mut user_name)?;
        user_name = user_name.trim().to_owned();

        if !user_name.is_empty() {
            Ok(user_name)
        } else if !proposed_name.is_empty() {
            Ok(proposed_name)
        } else {
            println!("The provided name cannot be empty.");
            self.get_fresh_project_name(cwd)
        }
    }

    fn ask_about_unassociated(
        &self,
        unassociated: &'a[String],
    ) -> Result<UserChoice<'a>, Box<dyn Error>> {
        println!("0) Set up a fresh project");
        for (i, project) in unassociated.iter().enumerate() {
            println!("{}) Associate with the project '{}'", i + 1, project);
        }

        println!("Which option do you select? (awaiting input...)");
        print!("> ");

        let mut choice = String::new();
        std::io::stdin().read_line(&mut choice)?;

        if choice == "0" {
            return Ok(UserChoice::Fresh);
        }

        for (i, project) in unassociated.iter().enumerate() {
            if choice == (i + 1).to_string() {
                return Ok(UserChoice::Existing(project));
            }
        }

        println!("Provided option {} is uncrecognized. Please choose one of the below or press CTRL+C to cancel.", choice);

        self.ask_about_unassociated(unassociated)
    }
}

enum UserChoice<'a> {
    Fresh,
    Existing(&'a str),
}