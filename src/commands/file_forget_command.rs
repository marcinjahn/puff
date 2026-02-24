use anyhow::{anyhow, bail, Result};
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    config::{locations::LocationsProvider, projects::ProjectsRetriever},
    fs_utils::is_symlink,
};

pub struct ForgetCommand<'a> {
    locations_provider: &'a LocationsProvider,
    projects_retriever: &'a ProjectsRetriever<'a>,
}

impl<'a> ForgetCommand<'a> {
    pub fn new(
        locations_provider: &'a LocationsProvider,
        projects_retriever: &'a ProjectsRetriever,
    ) -> ForgetCommand<'a> {
        ForgetCommand {
            locations_provider,
            projects_retriever,
        }
    }

    pub fn forget_file(
        &self,
        mut user_file: PathBuf,
        current_dir: &Path,
        delete_file: bool,
    ) -> Result<()> {
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
            bail!("The specified path is a directory. A file path is required.");
        }

        let file_name = user_file
            .file_name()
            .ok_or_else(|| anyhow!("Couldn't retrieve file name"))?;
        let user_dir = user_file
            .parent()
            .ok_or_else(|| anyhow!("Could not retrieve user's project directory"))?;

        let project_name = self
            .locations_provider
            .get_project_name_by_user_dir(user_dir)?;

        if !self.is_file_added(user_dir, &project_name, file_name)? {
            bail!("The specified file does not belong to any puff-managed project.");
        }

        if user_file.exists() && !is_symlink(&user_file)? {
            bail!("The file exists but is not a puff symlink. The managed version may reference a deleted file. Resolve the local file first, then re-run the command.");
        }

        // it's safe to remove the symlink at this point
        if user_file.exists() {
            fs::remove_file(&user_file)?;
        }

        if !delete_file {
            self.copy_file(user_dir, &project_name, file_name)?;
        }

        self.remove_managed_file(&project_name, file_name)?;

        println!("Removed {file_name:?} from project '{project_name}'.");
        Ok(())
    }

    /// Checks whether a file has even beed aded to puff
    fn is_file_added(
        &self,
        user_dir: &Path,
        project_name: &str,
        file_name: &OsStr,
    ) -> Result<bool> {
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
    ) -> Result<()> {
        let managed_dir = self.locations_provider.get_managed_dir(project_name);
        let managed_file = managed_dir.join(file_name);
        fs::copy(managed_file, user_dir.join(file_name))?;

        Ok(())
    }

    fn remove_managed_file(
        &self,
        project_name: &str,
        file_name: &OsStr,
    ) -> Result<()> {
        let managed_dir = self.locations_provider.get_managed_dir(project_name);
        let managed_file = managed_dir.join(file_name);
        fs::remove_file(managed_file)?;

        Ok(())
    }
}
