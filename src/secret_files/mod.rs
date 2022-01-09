use std::{path::Path, error::Error, fs::{self, File}};

use crate::{config::{app_config::AppConfigManager, locations}, error::AppError};

pub fn add_file(path: &Path) -> Result<String, Box<dyn Error>> {
    if !path.is_absolute() {
        return Err(Box::new(AppError(
            "The provided file path is not absolute".into(),
        )));
    } else if path.is_dir() {
        return Err(Box::new(AppError(
            "The provided path points at a directory. A file is expected (existing or not)".into(),
        )));
    }

    let containing_dir = path.parent();
    if containing_dir.is_none() {
        return Err(Box::new(AppError(
            "The provided file path does not have any parent".into(),
        )));
    }
    let containing_dir = containing_dir.unwrap();

    let config = AppConfigManager::new()?.get_config()?;
    
    let project_name = match config.projects.iter().find(|p| p.path == containing_dir) {
        None => {
            return Err(Box::new(AppError(
                "Parent directory of the provided file is not associated with any project known to conman. Did you initialize it with 'conman init'?".into(),
            )));
        },
        Some(project) => {
            &project.name
        }
    };

    let project_configs_dir = locations::get_project_config_path(project_name)?;

    if !project_configs_dir.exists() {
        // TODO: Some command like 'conman doctor' should be added to fix conman config issues
        return Err(Box::new(AppError(
            format!("conman is in corrupted state. A project called '{}' is defined in conman's config.json, however its project directory is missing", project_name),
        )));
    }

    if path.exists() {
        handle_existing_file(path, &project_configs_dir)?;
    } else {
        handle_new_file(path, &project_configs_dir)?;
    }


    Ok(project_name.clone())
}

fn handle_new_file(user_path: &Path, project_configs_path: &Path) -> Result<(), Box<dyn Error>> {
    let file_name = user_path.file_name().unwrap();
    let new_path = project_configs_path.join(file_name);
    File::create(&new_path)?;
    symlink::symlink_file(new_path, user_path)?;

    Ok(())
}

fn handle_existing_file(user_path: &Path, project_configs_path: &Path) -> Result<(), Box<dyn Error>> {
    let file_name = user_path.file_name().unwrap();
    let new_path = project_configs_path.join(file_name);

    fs::copy(&user_path, &new_path)?;
    fs::remove_file(user_path)?;

    symlink::symlink_file(new_path, user_path)?;

    Ok(())
}