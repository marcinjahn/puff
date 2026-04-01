use anyhow::{Result, bail};
use std::fs;

use crate::{
    config::{
        app_config::AppConfigManager,
        projects::{AssociatedProject, ManagedItem, ProjectDetails, ProjectsRetriever},
    },
    fs_utils::{copy_dir_recursive, get_backup_path, is_symlink, remove_dir_symlink},
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
    ) -> Result<()> {
        let project_details = self.projects_retriever.get_details(&name)?;

        if project_details.is_none() {
            bail!("Project '{name}' is not managed by puff.");
        }

        if !skip_confirmation
            && !confirm(format!(
                "Are you sure you want to remove the project '{name}'?"
            ))?
        {
            println!("Aborted.");
            return Ok(());
        }

        let project_details = project_details.unwrap();

        if let ProjectDetails::Associated(associated) = &project_details {
            if delete_files {
                self.remove_symlinks(associated)?;
            } else {
                self.replace_symlinks(associated)?;
            }
        }

        self.remove_managed_dir(&project_details)?;
        self.update_config(&project_details)?;

        if delete_files || project_details.info().items.is_empty() {
            println!("Project '{name}' removed.");
        } else {
            println!(
                "Project '{name}' removed. Managed files have been restored to the project directory."
            );
        }
        Ok(())
    }

    fn remove_managed_dir(&self, project_details: &ProjectDetails) -> Result<()> {
        fs::remove_dir_all(&project_details.info().managed_dir)?;
        Ok(())
    }

    fn remove_symlinks(&self, associated: &AssociatedProject) -> Result<()> {
        for item in &associated.info.items {
            let path = associated.user_dir.join(item.path());
            if path.symlink_metadata().is_err() {
                continue;
            }
            if is_symlink(&path)? {
                if item.is_directory() {
                    remove_dir_symlink(&path)?;
                } else {
                    fs::remove_file(&path)?;
                }
            }
        }
        Ok(())
    }

    fn replace_symlinks(&self, associated: &AssociatedProject) -> Result<()> {
        for item in &associated.info.items {
            let mut target_path = associated.user_dir.join(item.path());
            fs::create_dir_all(target_path.parent().unwrap())?;

            match item {
                ManagedItem::File(_) => {
                    if !is_symlink(&target_path)? {
                        target_path = get_backup_path(&target_path)?;
                    } else {
                        fs::remove_file(&target_path)?;
                    }
                    fs::copy(associated.info.managed_dir.join(item.path()), target_path)?;
                }
                ManagedItem::Directory(_) => {
                    if is_symlink(&target_path)? {
                        remove_dir_symlink(&target_path)?;
                    }
                    copy_dir_recursive(
                        &associated.info.managed_dir.join(item.path()),
                        &target_path,
                    )?;
                }
            }
        }
        Ok(())
    }

    fn update_config(&self, project_details: &ProjectDetails) -> Result<()> {
        self.app_config_manager
            .remove_project(&project_details.info().name)?;
        Ok(())
    }
}
