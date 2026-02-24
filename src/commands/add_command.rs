use crate::{
    config::locations::LocationsProvider,
    fs_utils::symlink_file,
    git_ignore::{GitIgnoreHandler, GitIgnoreResult},
};
use anyhow::{Result, anyhow, bail};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

/// Handler for the `puff add <file>` command
pub struct AddCommand<'a> {
    locations_provider: &'a LocationsProvider,
}

impl<'a> AddCommand<'a> {
    pub fn new(locations_provider: &LocationsProvider) -> AddCommand<'_> {
        AddCommand { locations_provider }
    }

    /// Adds new file to puff. A project needs to exist prior to invoking
    /// that command. Optionally the file may be added to .gitignore.
    pub fn add_file(
        &self,
        mut user_file: PathBuf,
        current_dir: &Path,
        add_to_git_ignore: bool,
    ) -> Result<()> {
        if !user_file.is_absolute() {
            user_file = current_dir.join(user_file);
        }

        if user_file.is_dir() {
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
        let managed_dir = self.locations_provider.get_managed_dir(&project_name);
        let relative_path = user_file.strip_prefix(&project_root)?;
        let managed_file = managed_dir.join(relative_path);

        if !managed_dir.exists() {
            // TODO: Some command like 'puff doctor' should be added to fix puff config issues
            bail!(
                "Corrupted state: project '{}' is registered in config.json but its directory is missing.",
                project_name
            );
        }

        fs::create_dir_all(managed_file.parent().unwrap())?;

        let mut message = String::from("");
        if user_file.exists() && managed_file.exists() {
            AddCommand::handle_two_files(&user_file, &managed_file);
        } else if !user_file.exists() && managed_file.exists() {
            AddCommand::handle_only_managed_exists(&managed_file, &user_file)?;
            message = "It was symlinked to an existing file managed by puff.".to_string();
        } else if user_file.exists() {
            AddCommand::handle_only_user_file_exists(&user_file, &managed_file)?;
        } else {
            AddCommand::handle_fresh_file(&user_file, &managed_file)?;
        }

        if add_to_git_ignore {
            let handler = GitIgnoreHandler::new();
            handler.add_to_git_ignore(
                user_dir,
                file_name
                    .to_str()
                    .ok_or_else(|| anyhow!("File name could not be parsed"))?,
            )?;
        }

        println!(
            "Added {:?} to project '{project_name}'. {message}",
            relative_path
        );

        Ok(())
    }

    /// Handles a case where a file being added already exists in puff (and not in
    /// user's directory).
    fn handle_only_managed_exists(managed_file: &Path, user_file: &Path) -> Result<()> {
        symlink_file(managed_file, user_file)?;
        Ok(())
    }

    /// Handles a situation where the file being added by the user already exists
    /// in both user's project directory and in puff's project directory. The following
    /// cases are covered: the file in user directory is a directory; the file in user
    /// directory is already a valid puff symlink; the files in puff and user's directory
    /// are totally different. In all these cases function returns 'true', which means the
    /// program should terminate.
    fn handle_two_files(user_file: &Path, managed_file: &Path) {
        if let Ok(symlink_path) = fs::read_link(user_file)
            && symlink_path == managed_file
        {
            println!("{:?} is already managed by puff. Nothing to do.", user_file);
        }

        let metadata = fs::metadata(user_file);
        if let Ok(metadata) = metadata {
            if metadata.file_type().is_dir() {
                println!("{:?} is a directory, not a file. Aborting.", user_file);
            } else {
                println!(
                    "Conflict: {:?} exists in both the project directory and puff's registry. \
                    Rename the local file and re-run the command to resolve the conflict.",
                    user_file
                );
            }
        } else {
            println!("Could not access {:?}. Aborting.", user_file);
        }
    }

    /// Handles a case where both user's directory and puff have no file. It will
    /// be created in puff and user's directory will have a symlink to it.
    fn handle_fresh_file(user_file: &Path, managed_file: &Path) -> Result<()> {
        File::create(managed_file)?;
        symlink_file(managed_file, user_file)?;

        Ok(())
    }

    /// Handles a case where a file being added already exists in user's directory.
    /// It will be moved to puff, and a softlink to it will be created in
    /// user's directory
    fn handle_only_user_file_exists(user_path: &Path, managed_file: &Path) -> Result<()> {
        fs::copy(user_path, managed_file)?;
        fs::remove_file(user_path)?;

        symlink_file(managed_file, user_path)?;

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
        let current_dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(puff_dir.path().join("configs/proj1")).unwrap();
        let locations_provider = LocationsProvider::new(puff_dir.path().to_path_buf());

        let user_file = current_dir.path().join("file");
        let config_file = puff_dir.path().join("config.json");
        let mut file = File::create(&config_file).unwrap();
        write!(file, "{{\"projects\":[]}}").unwrap();

        let sut = AddCommand::new(&locations_provider);

        let result = sut.add_file(user_file, current_dir.path(), false);

        assert!(result.is_err());
        // TODO: Use proper error kinds and check that
        let message = result.unwrap_err().to_string();
        print!("{}", message);
        assert!(message.contains("The current directory is not associated with any"));
    }

    #[test]
    fn add_file_fresh_scenario() {
        let puff_dir = tempfile::tempdir().unwrap();
        let current_dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(puff_dir.path().join("configs/proj1")).unwrap();
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

    #[test]
    fn add_file_in_subdirectory() {
        let puff_dir = tempfile::tempdir().unwrap();
        let project_root = tempfile::tempdir().unwrap();
        fs::create_dir_all(puff_dir.path().join("configs/proj1")).unwrap();
        let locations_provider = LocationsProvider::new(puff_dir.path().to_path_buf());

        let config_file = puff_dir.path().join("config.json");
        let mut file = File::create(&config_file).unwrap();
        write!(
            file,
            "{{\"projects\":[{{\"name\":\"proj1\", \"path\":\"{}\", \"id\":\"1\"}}]}}",
            project_root.path().to_str().unwrap()
        )
        .unwrap();

        let subdir = project_root.path().join("config");
        fs::create_dir_all(&subdir).unwrap();
        let user_file = subdir.join("secrets.env");

        let sut = AddCommand::new(&locations_provider);
        sut.add_file(user_file, project_root.path(), false).unwrap();

        let managed_file = puff_dir.path().join("configs/proj1/config/secrets.env");
        assert!(managed_file.exists());
    }

    #[test]
    fn add_file_from_subdirectory_cwd() {
        let puff_dir = tempfile::tempdir().unwrap();
        let project_root = tempfile::tempdir().unwrap();
        fs::create_dir_all(puff_dir.path().join("configs/proj1")).unwrap();
        let locations_provider = LocationsProvider::new(puff_dir.path().to_path_buf());

        let config_file = puff_dir.path().join("config.json");
        let mut file = File::create(&config_file).unwrap();
        write!(
            file,
            "{{\"projects\":[{{\"name\":\"proj1\", \"path\":\"{}\", \"id\":\"1\"}}]}}",
            project_root.path().to_str().unwrap()
        )
        .unwrap();

        let subdir = project_root.path().join("config");
        fs::create_dir_all(&subdir).unwrap();
        let user_file = subdir.join("secrets.env");

        let sut = AddCommand::new(&locations_provider);
        // CWD is the subdir, file path is relative
        sut.add_file(std::path::PathBuf::from("secrets.env"), &subdir, false)
            .unwrap();

        let managed_file = puff_dir.path().join("configs/proj1/config/secrets.env");
        assert!(managed_file.exists());

        let symlink_target = fs::read_link(&user_file).unwrap();
        assert_eq!(managed_file, symlink_target);
    }
}
