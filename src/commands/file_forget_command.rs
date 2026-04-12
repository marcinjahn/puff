use anyhow::{Result, anyhow, bail};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    config::locations::LocationsProvider,
    fs_utils::{copy_dir_recursive, is_symlink, remove_dir_symlink},
    managed_dirs::{self, PathClassification},
};

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

        let parent = user_file
            .parent()
            .ok_or_else(|| anyhow!("Could not retrieve user's project directory"))?;

        let (project_name, project_root) = self.locations_provider.find_project_for_path(parent)?;
        let managed_dir = self.locations_provider.get_managed_dir(&project_name);
        let relative_path = user_file.strip_prefix(&project_root)?;

        match managed_dirs::classify_path(&managed_dir, relative_path)? {
            PathClassification::IsManaged => {
                return self.forget_directory(
                    &user_file,
                    &project_name,
                    &managed_dir,
                    relative_path,
                    delete_file,
                );
            }
            PathClassification::InsideManaged(parent_managed) => {
                bail!(
                    "'{0}' is part of managed directory '{1}/'. \
                    Use 'puff forget {1}' to stop managing the entire directory.",
                    relative_path.display(),
                    parent_managed.display()
                );
            }
            PathClassification::Unmanaged => {}
        }

        // Original file forget logic
        if user_file.is_dir() && !delete_file {
            bail!("The specified path is a directory. A file path is required.");
        }

        let file_name = user_file
            .file_name()
            .ok_or_else(|| anyhow!("Couldn't retrieve file name"))?;

        if !self.is_file_added(&project_name, relative_path)? {
            bail!("The specified file does not belong to any puff-managed project.");
        }

        if user_file.exists() && !is_symlink(&user_file)? {
            bail!(
                "The file exists but is not a puff symlink. The managed version may reference a deleted file. Resolve the local file first, then re-run the command."
            );
        }

        if user_file.exists() {
            fs::remove_file(&user_file)?;
        }

        if !delete_file {
            self.copy_file(&user_file, &project_name, relative_path)?;
        }

        self.remove_managed_file(&project_name, relative_path)?;

        println!("Restored {file_name:?} in project '{project_name}'.");
        Ok(())
    }

    fn forget_directory(
        &self,
        user_path: &Path,
        project_name: &str,
        managed_dir: &Path,
        relative_path: &Path,
        delete_file: bool,
    ) -> Result<()> {
        let managed_target = managed_dir.join(relative_path);

        if !managed_target.exists() {
            bail!("The managed directory does not exist in puff's data store.");
        }

        // Remove the symlink
        if (user_path.exists() || user_path.symlink_metadata().is_ok()) && is_symlink(user_path)? {
            remove_dir_symlink(user_path)?;
        }

        if !delete_file {
            // Restore: copy directory back from managed store
            copy_dir_recursive(&managed_target, user_path)?;
        }

        // Remove from data store
        fs::remove_dir_all(&managed_target)?;

        // Remove from .puff_managed_dirs
        managed_dirs::remove_managed_dir(managed_dir, relative_path)?;

        let dir_name = relative_path.display();
        if delete_file {
            println!("Removed '{dir_name}/' from project '{project_name}'.");
        } else {
            println!("Restored '{dir_name}/' in project '{project_name}'.");
        }
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
        if let Some(parent) = managed_file.parent()
            && parent != managed_dir
        {
            let _ = fs::remove_dir(parent);
        }

        Ok(())
    }
}
