use crate::{
    config::{app_config::AppConfigManager, locations},
    error::AppError,
    fs_utils::{is_empty_dir, backup_file},
};
use std::{error::Error, fs, path::Path};

/// Initializes a project that already exists in conman's configs
/// directory. It updates conman's config file by adding that new project there.
pub fn init_project(name: &str, user_dir: &Path) -> Result<(), Box<dyn Error>> {
    let managed_dir = locations::get_managed_dir(name)?;
    if !managed_dir.exists() {
        return Err(Box::new(AppError(
            "The project folder does not exist in conman's configs".into(),
        )));
    }

    let manager = AppConfigManager::new()?;
    manager.add(name, user_dir)?;

    bring_in_existing_secrets(name, user_dir)?;

    Ok(())
}

/// Initializes the user's project directory with files managed by conman
fn bring_in_existing_secrets(project_name: &str, user_dir: &Path) -> Result<(), Box<dyn Error>> {
    let managed_dir = locations::get_managed_dir(project_name)?;
    if is_empty_dir(&managed_dir)? {
        return Ok(());
    }

    for file in managed_dir.read_dir()? {
        match file {
            Ok(file) => {
                handle_existing_file(&file.path(), &user_dir)?;
            }
            Err(_err) => {
                return Err(Box::new(AppError(
                    "The project already contains some files, but some of them could not be read"
                        .into(),
                )));
            }
        }
    }

    Ok(())
}

/// Sets up a single file managed by conman to be accessible in user's project
/// directory
fn handle_existing_file(managed_file: &Path, user_dir: &Path) -> Result<(), Box<dyn Error>> {
    let managed_file_name = managed_file.file_name();
    if managed_file_name.is_none() {
        return Err(Box::new(AppError(format!(
            "Existing file {:?} could not be read",
            managed_file
        ))));
    }
    let managed_file_name = managed_file_name.unwrap();

    let file_in_user_dir = user_dir.join(managed_file_name);
    if file_in_user_dir.exists() {
        let backup = backup_file(&file_in_user_dir)?;
        fs::remove_file(&file_in_user_dir)?;
        println!("Both the initialized directory and conman had the file {:?}. A backup of the file {:?} had been created under {}. Currently, {:?} points to the file that was present in conman. Remember to deal somehow with the backup file, probably you don't want it to end up in your remote repository (if you use it for this project).", 
            managed_file.file_name().unwrap(), 
            &file_in_user_dir, 
            backup.unwrap(), 
            file_in_user_dir.file_name().unwrap());
    }

    symlink::symlink_file(managed_file, user_dir)?;

    Ok(())
}