use crate::{
    config::{app_config::AppConfigManager, locations},
    error::AppError,
    fs_utils::is_empty_dir,
};
use std::{error::Error, fs, path::Path};

/// Initializes a project that already exists in conman's configs
/// directory. It updates conman's config file by adding that new project there.
pub fn init_project(name: &str, path: &Path) -> Result<(), Box<dyn Error>> {
    let project_config_path = locations::get_project_config_path(name)?;
    if !project_config_path.exists() {
        return Err(Box::new(AppError(
            "The project folder does not exist in conman's configs".into(),
        )));
    }

    let manager = AppConfigManager::new()?;
    manager.add(name, path)?;

    bring_in_existing_secrets(name, path)?;

    Ok(())
}

fn bring_in_existing_secrets(name: &str, path: &Path) -> Result<(), Box<dyn Error>> {
    let project_location = locations::get_project_config_path(name)?;
    if is_empty_dir(&project_location)? {
        return Ok(());
    }

    for file in path.read_dir()? {
        match file {
            Ok(file) => {
                handle_existing_file(&file.path(), &path)?;
            }
            Err(err) => {
                return Err(Box::new(AppError(
                    "The project already contains some files, but some of them could not be read"
                        .into(),
                )));
            }
        }
    }

    Ok(())
}

fn handle_existing_file(file_path: &Path, user_path: &Path) -> Result<(), Box<dyn Error>> {
    let file_name = file_path.file_name();
    if file_name.is_none() {
        return Err(Box::new(AppError(format!(
            "Existing file {:?} could not be read",
            file_path
        ))));
    }
    let file_name = file_name.unwrap();

    let file_in_user_dir = user_path.join(file_name);
    if file_in_user_dir.exists() {
        let backup = backup_file(&file_in_user_dir)?;
        fs::remove_file(&file_in_user_dir)?;
        println!("Both the initialized directory and conman had the file {:?}. A backup of the file {:?} had been created under {}. Currently, {:?} points to the file that was present in conman. Remember to deal somehow with the backup file, probably you don't want it to end up in your remote repository (if you use it for this project).", 
            file_path.file_name().unwrap(), 
            &file_in_user_dir, 
            backup.unwrap(), 
            file_in_user_dir.file_name().unwrap());
    }

    // TODO: fix naming of user_path and project_path. It's quite confusing now
    symlink::symlink_file(new_path, user_path)?;

    Ok(())
}

fn backup_file(file_path: &Path) -> Result<Option<String>, Box<dyn Error>> {
    let mut path_string = file_path.to_str().unwrap().to_owned();
    let mut bak = path_string + ".bak";
    let mut backup_path = Path::new(&bak);

    while backup_path.exists() {
        path_string = backup_path.to_str().unwrap().to_owned();
        bak = path_string + "1";
        backup_path = Path::new(&bak);
    }

    fs::copy(file_path, backup_path)?;

    Ok(Some(backup_path.to_str().unwrap().to_string()))
}
