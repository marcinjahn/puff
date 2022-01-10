use std::{path::Path, error::Error, fs::{self, File}};

use crate::{config::{locations}, error::AppError};

pub fn add_file(user_file: &Path) -> Result<String, Box<dyn Error>> {
    let containing_dir = user_file.parent();
    if containing_dir.is_none() {
        return Err(Box::new(AppError(
            "The provided file path does not have any parent".into(),
        )));
    }
    let containing_dir = containing_dir.unwrap();
    let project_name = locations::get_project_name_by_user_dir(containing_dir)?;
    let managed_dir = locations::get_managed_dir(&project_name)?;



    if user_file.exists() {
        handle_existing_file(user_file, &managed_dir)?;
    } else {
        handle_new_file(user_file, &managed_dir)?;
    }


    Ok(project_name.clone())
}

fn handle_new_file(user_file: &Path, managed_dir: &Path) -> Result<(), Box<dyn Error>> {
    let file_name = user_file.file_name().unwrap();
    let managed_file = managed_dir.join(file_name);

    File::create(&managed_file)?;
    symlink::symlink_file(managed_file, user_file)?;

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