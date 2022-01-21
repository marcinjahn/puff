use std::{
    error::Error,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    config::{locations::LocationsProvider, projects::ProjectsRetriever},
    error::AppError, fs_utils::is_symlink,
};

pub struct RmCommand<'a> {
    locations_provider: &'a LocationsProvider,
    projects_retriever: &'a ProjectsRetriever<'a>,
}

impl<'a> RmCommand<'a> {
    pub fn new(locations_provider: &'a LocationsProvider, projects_retriever: &'a ProjectsRetriever) -> RmCommand<'a> {
        RmCommand { locations_provider, projects_retriever}
    }

    pub fn remove_file(
        &self,
        mut user_file: PathBuf,
        current_dir: &Path,
        delete_file: bool,
    ) -> Result<(), Box<dyn Error>> {
        if !user_file.is_absolute() {
            user_file = current_dir.join(user_file);
        }

        // if !user_file.exists() {
        //     return Err(Box::new(AppError(
        //         "The provided file does not exist"
        //             .into(),
        //     )));
        // }

        if user_file.is_dir() && !delete_file {
            return Err(Box::new(AppError(
                "The provided path points at a directory. A file is expected".into(),
            )));
        }

        let file_name = user_file.file_name().ok_or("Couldn't retrieve file name")?;
        let user_dir = user_file
            .parent()
            .ok_or("Could not retrieve user's project directory")?;

        let project_name = self
            .locations_provider
            .get_project_name_by_user_dir(user_dir)?;

        if !self.is_file_added(user_dir, &project_name, file_name)? {
            return Err(Box::new(AppError(
                "The provided file does not belong to any associated project known to conman"
                    .into(),
            )));
        }

        if user_file.exists() && !is_symlink(&user_file)? {
            return Err(Box::new(AppError(
                "The provided file is not managed by conman. Conman must have been configured with some previous version of that file that has been deleted since then. Deal with that file first and then invoke the \"rm\" command again to remove the version that conman has stored."
                    .into(),
            )));
        }

        // it's safe to remove the symlink at this point
        if user_file.exists() {
            fs::remove_file(&user_file)?;
        }

        if !delete_file {
            self.copy_file(user_dir, &project_name, file_name)?;
        }

        self.remove_managed_file(&project_name, file_name)?;

        println!("The file {file_name:?} has been removed from the project {project_name}");
        Ok(())
    }

    /// Checks whether a file has even beed aded to conman
    fn is_file_added(
        &self,
        user_dir: &Path,
        project_name: &str,
        file_name: &OsStr,
    ) -> Result<bool, Box<dyn Error>> {
        let is_associated = self.projects_retriever.is_associated(user_dir)?;

        if !is_associated {
            return Ok(false);
        }

        let managed_path = self.locations_provider.get_managed_dir(project_name);
        if !managed_path.join(file_name).exists() {
            return Ok(false);
        }

        Ok(true)
    }

    fn copy_file(
        &self,
        user_dir: &Path,
        project_name: &str,
        file_name: &OsStr,
    ) -> Result<(), Box<dyn Error>> {
        let managed_dir = self.locations_provider.get_managed_dir(project_name);
        let managed_file = managed_dir.join(file_name);
        fs::copy(managed_file, user_dir.join(file_name))?;

        Ok(())
    }

    fn remove_managed_file(&self, project_name: &str, file_name: &OsStr) -> Result<(), Box<dyn Error>> {
        let managed_dir = self.locations_provider.get_managed_dir(project_name);
        let managed_file = managed_dir.join(file_name);
        fs::remove_file(managed_file)?;

        Ok(())
    }
}
