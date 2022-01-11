use std::{
    env,
    error::Error,
    fs::{self, File},
    path::{Path, PathBuf},
};

use crate::{config::locations, error::AppError};

pub fn add_file(mut user_file: PathBuf) -> Result<(), Box<dyn Error>> {
    if !user_file.is_absolute() {
        let cwd = env::current_dir()?;
        user_file = cwd.join(user_file);
    }

    if user_file.is_dir() {
        return Err(Box::new(AppError(
            "The provided path points at a directory. A file is expected (existing or not)".into(),
        )));
    }

    let file_name = user_file.file_name().ok_or("Couldn't retrieve file name")?;
    let user_dir = user_file
        .parent()
        .ok_or("Could not retrieve user's project directory")?;
    let project_name = locations::get_project_name_by_user_dir(&user_dir)?;
    let managed_dir = locations::get_managed_dir(&project_name)?;
    let managed_file = &managed_dir.join(Path::new(file_name));

    if !managed_dir.exists() {
        // TODO: Some command like 'conman doctor' should be added to fix conman config issues
        return Err(Box::new(AppError(
            format!("conman is in corrupted state. A project called '{}' is defined in conman's config.json, however its project directory is missing", project_name),
        )));
    }

    if user_file.exists()
        && managed_file.exists()
        && handle_two_files(&user_file, &managed_file)
    {
        return Ok(());
    }

    if !user_file.exists() && managed_file.exists() {
        handle_only_managed_exists(managed_file, &user_file)?;
    }

    if user_file.exists() {
        handle_only_user_file_exists(&user_file, &managed_dir)?;
    } else {
        handle_happy_path(&user_file, &managed_dir)?;
    }

    println!(
        "The file {:?} has been added to the project named '{}'",
        file_name, project_name
    );

    Ok(())
}

/// Handles a case where a file being added already exists in conamn (and not in
/// user's directory).
fn handle_only_managed_exists(managed_file: &Path, user_file: &Path) -> Result<(), Box<dyn Error>> {
    symlink::symlink_file(managed_file, user_file)?;
    Ok(())
}

/// Handles a situation where the file being added by the user already exists
/// in both user's project directory and in conman's project directory. The following
/// cases are covered: the file in user directory is a directory; the file in user
/// directory is already a valid conman symlink; the files in conman and user's directory
/// are totally different. In all these cases function returns 'true', which means the
/// program should terminate.
fn handle_two_files(user_file: &Path, managed_file: &Path) -> bool {
    if let Ok(symlink_path) = fs::read_link(&user_file) {
        if symlink_path == managed_file {
            println!("The file {:?} has already been configured with conman and there's nothing more to do.", user_file);
            return true;
        }
    }

    let metadata = fs::metadata(user_file);
    if let Ok(metadata) = metadata {
        if metadata.file_type().is_dir() {
            println!(
                "{:?} is a directory, which is invalid. Aborting.",
                user_file
            );
            return true;
        } else {
            println!(
                "Both the project directory and conman have the file with the same name. Conman can't resolve that conflict. One way around it is to rename the file {:?} and run the command again. Conman will then set up {:?} to point to the file stored within conman registry. You will be able to modify it.",
                user_file, user_file
            );
            return true;
        }
    }

    false
}

/// Handles a case where both user's directory and conamn have no file.  It will
/// be created in conman and user's directory will have a symlink to it.
fn handle_happy_path(user_file: &Path, managed_dir: &Path) -> Result<(), Box<dyn Error>> {
    let file_name = user_file.file_name().unwrap();
    let managed_file = managed_dir.join(file_name);

    File::create(&managed_file)?;
    symlink::symlink_file(managed_file, user_file)?;

    Ok(())
}

/// Handles a case where a file being added already exists in user's directory.
/// It will be moved to conman, and a softlink to it will be created in
/// user's directory
fn handle_only_user_file_exists(user_path: &Path, project_configs_path: &Path) -> Result<(), Box<dyn Error>> {
    let file_name = user_path.file_name().unwrap();
    let new_path = project_configs_path.join(file_name);

    fs::copy(&user_path, &new_path)?;
    fs::remove_file(user_path)?;

    symlink::symlink_file(new_path, user_path)?;

    Ok(())
}
