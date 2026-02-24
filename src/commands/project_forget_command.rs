use std::{error::Error, fs};

use crate::{
    config::{
        app_config::AppConfigManager,
        projects::{ProjectDetails, ProjectsRetriever},
    },
    fs_utils::{get_backup_path, is_symlink},
    io_utils::confirm,
};

pub struct ProjectForgetCommand<'a> {
    projects_retriever: &'a ProjectsRetriever<'a>,
    app_config_manager: &'a AppConfigManager,
}

impl<'a> ProjectForgetCommand<'a> {
    pub fn new(
        projects_retriever: &'a ProjectsRetriever,
        app_config_manager: &'a AppConfigManager,
    ) -> ProjectForgetCommand<'a> {
        ProjectForgetCommand {
            projects_retriever,
            app_config_manager,
        }
    }

    pub fn forget_project(
        &self,
        name: String,
        delete_files: bool,
        skip_confirmation: bool,
    ) -> Result<(), Box<dyn Error>> {
        let project_details = self.projects_retriever.get_details(&name)?;

        if project_details.is_none() {
            println!("Project called '{name}' is not managed by puff, it couldn't be removed");
            return Ok(());
        }

        if !skip_confirmation
            && !confirm(format!(
                "Are you sure you want to remove the project '{name}'?"
            ))?
        {
            println!("Removal aborted");
            return Ok(());
        }

        let project_details = project_details.unwrap();

        if delete_files {
            self.remove_symlinks(&project_details)?;
        } else {
            self.replace_symlinks(&project_details)?;
        }

        self.remove_managed_dir(&project_details)?;
        self.update_config(&project_details)?;

        if delete_files || project_details.files.is_empty() {
            println!("Project '{name}' has been removed");
        } else {
            // let files = project_details
            //     .files
            //     .iter()
            //     .map(|f| f.to_str().unwrap().to_owned())
            //     .collect::<Vec<String>>()
            //     .join(", ");

            println!("Project '{name}' has been removed. The files have been restored.");
        }
        Ok(())
    }

    fn remove_managed_dir(&self, project_details: &ProjectDetails) -> Result<(), Box<dyn Error>> {
        fs::remove_dir_all(&project_details.managed_dir)?;
        Ok(())
    }

    fn remove_symlinks(&self, project_details: &ProjectDetails) -> Result<(), Box<dyn Error>> {
        for file_name in &project_details.files {
            let path = project_details.user_dir.as_ref().unwrap().join(file_name);
            if is_symlink(&path)? {
                fs::remove_file(path)?;
            }
        }

        Ok(())
    }

    fn replace_symlinks(&self, project_details: &ProjectDetails) -> Result<(), Box<dyn Error>> {
        if !project_details.user_dir.is_some() {
            return Ok(());
        }
        let user_path = project_details.user_dir.as_ref().unwrap();

        for file in &project_details.files {
            let mut target_path = user_path.join(file);
            if !is_symlink(user_path)? {
                target_path = get_backup_path(user_path)?;
            } else {
                fs::remove_file(&target_path)?;
            }

            fs::copy(project_details.managed_dir.join(file), target_path)?;
        }

        Ok(())
    }

    fn update_config(&self, project_details: &ProjectDetails) -> Result<(), Box<dyn Error>> {
        self.app_config_manager
            .remove_project(&project_details.name)?;

        Ok(())
    }
}
