use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use crate::{config::locations, error::AppError, secret_files};

pub fn add_file(mut user_file: PathBuf) -> Result<(), Box<dyn Error>> {
    // TODO: Handle a case when user tries to add a file that is already managed by conman - in such a case, do nothing

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
        && handle_conflicting_files(&user_file, &managed_file)
    {
        return Ok(());
    }

    // TODO: Handle situation where managed_file exists and user_file doesn't
    // The app should then create a link to existing managed_file and inform the user about it.

    let project_name = secret_files::add_file(&user_file)?;

    println!(
        "The file {:?} has been added to the project named '{}'",
        file_name, project_name
    );

    Ok(())
}

/// Tries to handle a situation where the file being added by the user already exists
/// in both user's project directory and in conman's project directory. The following
/// situations are covered: the file in user directory is a directory; the file in user
/// directory is already a valid conman symlink; the files in conman and user's directory
/// are totally different. In all these cases function returns 'true', which means the
/// program should terminate.
fn handle_conflicting_files(user_file: &Path, managed_file: &Path) -> bool {
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
