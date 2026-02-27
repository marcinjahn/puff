use anyhow::{Result, anyhow, bail};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{config::locations::LocationsProvider, fs_utils::is_symlink};

pub struct ForgetCommand<'a> {
    locations_provider: &'a LocationsProvider,
}

impl<'a> ForgetCommand<'a> {
    pub fn new(locations_provider: &'a LocationsProvider) -> ForgetCommand<'a> {
        ForgetCommand { locations_provider }
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

        if user_file.is_dir() && !delete_file {
            bail!("The specified path is a directory. A file path is required.");
        }

        let file_name = user_file
            .file_name()
            .ok_or_else(|| anyhow!("Couldn't retrieve file name"))?;
        let user_dir = user_file
            .parent()
            .ok_or_else(|| anyhow!("Could not retrieve user's project directory"))?;

        let (project_name, project_root) =
            self.locations_provider.find_project_for_path(user_dir)?;
        let relative_path = user_file.strip_prefix(&project_root)?;

        if !self.is_file_added(&project_name, relative_path)? {
            bail!("The specified file does not belong to any puff-managed project.");
        }

        if user_file.exists() && !is_symlink(&user_file)? {
            bail!(
                "The file exists but is not a puff symlink. The managed version may reference a deleted file. Resolve the local file first, then re-run the command."
            );
        }

        // it's safe to remove the symlink at this point
        if user_file.exists() {
            fs::remove_file(&user_file)?;
        }

        if !delete_file {
            self.copy_file(&user_file, &project_name, relative_path)?;
        }

        self.remove_managed_file(&project_name, relative_path)?;

        println!("Removed {file_name:?} from project '{project_name}'.");
        Ok(())
    }

    fn is_file_added(&self, project_name: &str, relative_path: &Path) -> Result<bool> {
        let managed_path = self.locations_provider.get_managed_dir(project_name);
        Ok(managed_path.join(relative_path).exists())
    }

    fn copy_file(&self, user_file: &Path, project_name: &str, relative_path: &Path) -> Result<()> {
        let managed_dir = self.locations_provider.get_managed_dir(project_name);
        let managed_file = managed_dir.join(relative_path);
        fs::create_dir_all(user_file.parent().unwrap())?;
        fs::copy(managed_file, user_file)?;

        Ok(())
    }

    fn remove_managed_file(&self, project_name: &str, relative_path: &Path) -> Result<()> {
        let managed_dir = self.locations_provider.get_managed_dir(project_name);
        let managed_file = managed_dir.join(relative_path);
        fs::remove_file(&managed_file)?;

        // clean up empty subdirectory (ignore errors — dir may be non-empty or be the root)
        if let Some(parent) = managed_file.parent() {
            if parent != managed_dir {
                let _ = fs::remove_dir(parent);
            }
        }

        Ok(())
    }
}
