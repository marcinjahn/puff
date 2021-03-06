use crate::{config::locations::LocationsProvider, error::AppError, git_ignore::{GitIgnoreHandler, GitIgnoreResult}};
use std::{
    error::Error,
    fs::{self, File},
    path::{Path, PathBuf},
};

/// Handler for the `puff add <file>` command
pub struct AddCommand<'a> {
    locations_provider: &'a LocationsProvider,
}

impl<'a> AddCommand<'a> {
    pub fn new(locations_provider: &LocationsProvider) -> AddCommand {
        AddCommand { locations_provider }
    }

    /// Adds new file to puff. A project needs to exist prior to invoking
    /// that command. Optionally the file may be added to .gitignore.
    pub fn add_file(
        &self,
        mut user_file: PathBuf,
        current_dir: &Path,
        add_to_git_ignore: bool,
    ) -> Result<(), Box<dyn Error>> {
        if !user_file.is_absolute() {
            user_file = current_dir.join(user_file);
        }

        if user_file.is_dir() {
            return Err(Box::new(AppError(
                "The provided path points at a directory. A file is expected (existing or not)"
                    .into(),
            )));
        }

        let file_name = user_file.file_name().ok_or("Couldn't retrieve file name")?;
        let user_dir = user_file
            .parent()
            .ok_or("Could not retrieve user's project directory")?;
        let project_name = self
            .locations_provider
            .get_project_name_by_user_dir(user_dir)?;
        let managed_dir = self.locations_provider.get_managed_dir(&project_name);
        let managed_file = &managed_dir.join(Path::new(file_name));

        if !managed_dir.exists() {
            // TODO: Some command like 'puff doctor' should be added to fix puff config issues
            return Err(Box::new(AppError(
                format!("puff is in corrupted state. A project called '{}' is defined in puff's config.json, however its project directory is missing", project_name),
            )));
        }

        let mut message = String::from("");
        if user_file.exists() && managed_file.exists() {
            AddCommand::handle_two_files(&user_file, managed_file);
        }
        else if !user_file.exists() && managed_file.exists() {
            AddCommand::handle_only_managed_exists(managed_file, &user_file)?;
            message = format!("It was symlinked to an existing file managed by puff.");
        }
        else if user_file.exists() {
            AddCommand::handle_only_user_file_exists(&user_file, &managed_dir)?;
            message = format!("File's content has been persisted.");
        } else {
            AddCommand::handle_fresh_file(&user_file, &managed_dir)?;
        }

        let mut git_ignore_result: Option<GitIgnoreResult> = None;
        if add_to_git_ignore {
            let handler = GitIgnoreHandler::new();
            git_ignore_result = Some(handler.add_to_git_ignore(
                user_dir,
                file_name.to_str().ok_or("File name could not be parsed")?)?,
            );
        }

        println!(
            "The file {file_name:?} has been added to the project named '{project_name}'. {message}"
        );
        if let Some(git_ignore_result) = git_ignore_result {
            println!(
                ".gitignore file has been {git_ignore_result}"
            );
        }

        Ok(())
    }

    /// Handles a case where a file being added already exists in puff (and not in
    /// user's directory).
    fn handle_only_managed_exists(
        managed_file: &Path,
        user_file: &Path,
    ) -> Result<(), Box<dyn Error>> {
        symlink::symlink_file(managed_file, user_file)?;
        Ok(())
    }

    /// Handles a situation where the file being added by the user already exists
    /// in both user's project directory and in puff's project directory. The following
    /// cases are covered: the file in user directory is a directory; the file in user
    /// directory is already a valid puff symlink; the files in puff and user's directory
    /// are totally different. In all these cases function returns 'true', which means the
    /// program should terminate.
    fn handle_two_files(user_file: &Path, managed_file: &Path) {
        if let Ok(symlink_path) = fs::read_link(&user_file) {
            if symlink_path == managed_file {
                println!("The file {:?} has already been configured with puff and there's nothing more to do.", user_file);
            }
        }

        let metadata = fs::metadata(user_file);
        if let Ok(metadata) = metadata {
            if metadata.file_type().is_dir() {
                println!(
                    "{:?} is a directory, which is invalid. Aborting.",
                    user_file
                );
            } else {
                println!(
                    "Both the project directory and puff have the file with the same name. puff can't resolve that conflict. One way around it is to rename the file {:?} and run the command again. puff will then set up {:?} to point to the file stored within puff registry. You will be able to modify it.",
                    user_file, user_file
                );
            }
        } else {
            println!("The file {:?} could not be accessed. Aborting.", user_file);
        }
    }

    /// Handles a case where both user's directory and conamn have no file.  It will
    /// be created in puff and user's directory will have a symlink to it.
    fn handle_fresh_file(user_file: &Path, managed_dir: &Path) -> Result<(), Box<dyn Error>> {
        let file_name = user_file.file_name().unwrap();
        let managed_file = managed_dir.join(file_name);

        File::create(&managed_file)?;
        symlink::symlink_file(managed_file, user_file)?;

        Ok(())
    }

    /// Handles a case where a file being added already exists in user's directory.
    /// It will be moved to puff, and a softlink to it will be created in
    /// user's directory
    fn handle_only_user_file_exists(
        user_path: &Path,
        project_configs_path: &Path,
    ) -> Result<(), Box<dyn Error>> {
        let file_name = user_path.file_name().unwrap();
        let new_path = project_configs_path.join(file_name);

        fs::copy(&user_path, &new_path)?;
        fs::remove_file(user_path)?;

        symlink::symlink_file(new_path, user_path)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};

    use super::AddCommand;
    use crate::config::locations::LocationsProvider;
    use std::io::Write;

    #[test]
    fn add_file_when_project_does_not_exist() {
        let puff_dir = tempfile::tempdir().unwrap();
        let locations_provider = LocationsProvider::new(puff_dir.path().to_path_buf());

        let sut = AddCommand::new(&locations_provider);

        let current_dir = tempfile::tempdir().unwrap();
        let user_file = current_dir.path().join("file");
        let mut file = File::create(&user_file).unwrap();
        write!(file, "some content").unwrap();
        let result = sut.add_file(user_file, current_dir.path(), false);

        assert!(result.is_err());
        // TODO: Use proper error kinds and check that
        let message = (*result.err().unwrap()).to_string();
        assert!(message.contains("initialization failed"));
    }

    #[test]
    fn add_file_fresh_scenario() {
        let puff_dir = tempfile::tempdir().unwrap();
        let current_dir = tempfile::tempdir().unwrap();
        let _project_dir = fs::create_dir_all(puff_dir.path().join("configs/proj1")).unwrap();
        let locations_provider = LocationsProvider::new(puff_dir.path().to_path_buf());
        let user_file = current_dir.path().join("file");
        let config_file = puff_dir.path().join("config.json");
        let mut file = File::create(&config_file).unwrap();
        write!(
            file,
            "{{\"projects\":[{{\"name\":\"proj1\", \"path\":\"{}\", \"id\":\"1\"}}]}}",
            current_dir.path().to_str().unwrap()
        )
        .unwrap();

        let sut = AddCommand::new(&locations_provider);

        sut.add_file(user_file, current_dir.path(), false).unwrap();
    }
}
