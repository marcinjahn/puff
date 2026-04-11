use crate::{
    config::locations::LocationsProvider,
    fs_utils::{copy_dir_recursive, symlink_dir, symlink_file},
    git_ignore::GitIgnoreHandler,
    managed_dirs,
};
use anyhow::{Result, anyhow, bail};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

/// Handler for the `puff add <path>` command
pub struct AddCommand<'a> {
    locations_provider: &'a LocationsProvider,
}

impl<'a> AddCommand<'a> {
    pub fn new(locations_provider: &LocationsProvider) -> AddCommand<'_> {
        AddCommand { locations_provider }
    }

    pub fn add_file(
        &self,
        mut user_file: PathBuf,
        current_dir: &Path,
        add_to_git_ignore: bool,
        force_dir: bool,
    ) -> Result<()> {
        if !user_file.is_absolute() {
            user_file = current_dir.join(user_file);
        }

        // Detect trailing separator before normalization strips it — a trailing
        // slash is a signal that the user intends a directory.
        let had_trailing_sep = {
            let raw = user_file.as_os_str().to_string_lossy();
            raw.ends_with('/') || raw.ends_with(std::path::MAIN_SEPARATOR)
        };

        // Normalize the path (strip trailing slashes, redundant `.` segments, etc.)
        // so that downstream symlink creation doesn't fail on e.g. "mydir/".
        user_file = user_file.components().collect();

        let is_dir = if user_file.exists() {
            if user_file.is_dir() {
                if user_file.is_symlink() {
                    bail!("The specified path is already a symlink to a directory.");
                }
                true
            } else {
                if force_dir {
                    bail!("--dir was specified but the path exists and is a file.");
                }
                false
            }
        } else {
            if had_trailing_sep && !force_dir {
                bail!(
                    "Directory '{}' does not exist. Use --dir to create a new managed directory.",
                    user_file.display()
                );
            }
            force_dir || had_trailing_sep
        };

        if is_dir {
            self.add_directory(user_file, add_to_git_ignore)
        } else {
            self.add_single_file(user_file, add_to_git_ignore)
        }
    }

    fn resolve_project(&self, path: &Path) -> Result<(String, PathBuf, PathBuf, PathBuf)> {
        let parent = path
            .parent()
            .ok_or_else(|| anyhow!("Could not retrieve parent directory"))?;
        let (project_name, project_root) =
            self.locations_provider.find_project_for_path(parent)?;
        let managed_dir = self.locations_provider.get_managed_dir(&project_name);
        let relative_path = path.strip_prefix(&project_root)?.to_path_buf();

        if !managed_dir.exists() {
            bail!(
                "Corrupted state: project '{}' is registered in config.json but its directory is missing.",
                project_name
            );
        }

        Ok((project_name, project_root, managed_dir, relative_path))
    }

    fn add_directory(
        &self,
        user_path: PathBuf,
        add_to_git_ignore: bool,
    ) -> Result<()> {
        let (project_name, project_root, managed_dir, relative_path) = self.resolve_project(&user_path)?;
        if let Some(parent_managed) = managed_dirs::is_inside_managed_dir(&managed_dir, &relative_path)? {
            bail!(
                "'{0}' is already managed as part of directory '{1}'. \
                Use 'puff forget {1}' to stop managing the directory first.",
                relative_path.display(),
                parent_managed.display()
            );
        }

        let managed_target = managed_dir.join(&relative_path);

        if user_path.exists() {
            self.absorb_existing_directory(&user_path, &managed_dir, &managed_target, &relative_path)?;
        } else {
            fs::create_dir_all(&managed_target)?;
        }

        // Create directory symlink
        if user_path.exists() || user_path.symlink_metadata().is_ok() {
            // should have been removed in absorb; safety check
            if user_path.is_symlink() {
                fs::remove_file(&user_path)?;
            }
        }
        fs::create_dir_all(user_path.parent().unwrap())?;
        symlink_dir(&managed_target, &user_path)?;

        managed_dirs::add_managed_dir(&managed_dir, &relative_path)?;

        if add_to_git_ignore {
            let handler = GitIgnoreHandler::new();
            let dir_name = relative_path.display().to_string();
            let gitignore_entry = if dir_name.ends_with('/') {
                dir_name
            } else {
                format!("{}/", dir_name)
            };
            handler.add_to_git_ignore(
                &project_root,
                &gitignore_entry,
            )?;
        }

        println!(
            "Added {:?} (directory) to project '{project_name}'.",
            relative_path
        );

        Ok(())
    }

    fn absorb_existing_directory(
        &self,
        user_path: &Path,
        managed_dir: &Path,
        managed_target: &Path,
        relative_path: &Path,
    ) -> Result<()> {
        fs::create_dir_all(managed_target)?;

        // Walk the user directory. For each entry:
        // - If it's a symlink pointing into managed_dir (individually managed file), remove the symlink
        //   (the file is already in the data store; move it into the directory's managed location)
        // - Otherwise, copy it into the managed target
        self.absorb_dir_recursive(user_path, managed_dir, managed_target, relative_path)?;

        // Remove the original directory
        fs::remove_dir_all(user_path)?;

        Ok(())
    }

    fn absorb_dir_recursive(
        &self,
        user_dir: &Path,
        managed_dir: &Path,
        managed_target: &Path,
        relative_path: &Path,
    ) -> Result<()> {
        for entry in fs::read_dir(user_dir)? {
            let entry = entry?;
            let entry_path = entry.path();
            let entry_name = entry.file_name();
            let sub_managed = managed_target.join(&entry_name);

            if entry_path.is_symlink() {
                if let Ok(link_target) = fs::read_link(&entry_path) {
                    if link_target.starts_with(managed_dir) {
                        if !sub_managed.exists() {
                            if link_target.is_dir() {
                                copy_dir_recursive(&link_target, &sub_managed)?;
                            } else {
                                fs::copy(&link_target, &sub_managed)?;
                            }
                        }
                        // Remove the old individually managed file only if it's at a different location
                        if link_target != sub_managed {
                            if link_target.is_dir() {
                                let _ = fs::remove_dir_all(&link_target);
                            } else {
                                let _ = fs::remove_file(&link_target);
                            }
                            // Clean up empty parent dirs up to managed_dir
                            let mut parent = link_target.parent();
                            while let Some(p) = parent {
                                if p == managed_dir { break; }
                                if fs::remove_dir(p).is_err() { break; }
                                parent = p.parent();
                            }
                        }
                        continue;
                    }
                }
                // Symlink not pointing to our managed dir — copy the target
                if entry_path.is_dir() {
                    copy_dir_recursive(&entry_path, &sub_managed)?;
                } else if entry_path.exists() {
                    fs::copy(&entry_path, &sub_managed)?;
                }
            } else if entry_path.is_dir() {
                self.absorb_dir_recursive(&entry_path, managed_dir, &sub_managed, &relative_path.join(&entry_name))?;
            } else {
                if !sub_managed.exists() {
                    fs::create_dir_all(sub_managed.parent().unwrap())?;
                    fs::copy(&entry_path, &sub_managed)?;
                }
            }
        }
        Ok(())
    }

    fn add_single_file(
        &self,
        user_file: PathBuf,
        add_to_git_ignore: bool,
    ) -> Result<()> {
        let file_name = user_file
            .file_name()
            .ok_or_else(|| anyhow!("Couldn't retrieve file name"))?;
        let user_dir = user_file
            .parent()
            .ok_or_else(|| anyhow!("Could not retrieve user's project directory"))?;
        let (project_name, _project_root, managed_dir, ref relative_path) = self.resolve_project(&user_file)?;

        if let Some(parent_managed) = managed_dirs::is_inside_managed_dir(&managed_dir, relative_path)? {
            bail!(
                "'{0}' is already managed as part of directory '{1}/'. \
                The file is accessible through the directory symlink.",
                relative_path.display(),
                parent_managed.display()
            );
        }

        let managed_file = managed_dir.join(relative_path);

        fs::create_dir_all(managed_file.parent().unwrap())?;

        if user_file.exists() && managed_file.exists() {
            return AddCommand::handle_two_files(&user_file, &managed_file);
        }

        let mut message = String::from("");
        if !user_file.exists() && managed_file.exists() {
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

    fn handle_only_managed_exists(managed_file: &Path, user_file: &Path) -> Result<()> {
        symlink_file(managed_file, user_file)?;
        Ok(())
    }

    fn handle_two_files(user_file: &Path, managed_file: &Path) -> Result<()> {
        if let Ok(symlink_path) = fs::read_link(user_file)
            && symlink_path == managed_file
        {
            println!("{:?} is already managed by puff. Nothing to do.", user_file);
            return Ok(());
        }

        let metadata = fs::metadata(user_file);
        if let Ok(metadata) = metadata {
            if metadata.file_type().is_dir() {
                bail!("{:?} is a directory, not a file.", user_file);
            } else {
                bail!(
                    "Conflict: {:?} exists in both the project directory and puff's registry. \
                    Rename the local file and re-run the command to resolve the conflict.",
                    user_file
                );
            }
        } else {
            bail!("Could not access {:?}.", user_file);
        }
    }

    fn handle_fresh_file(user_file: &Path, managed_file: &Path) -> Result<()> {
        File::create(managed_file)?;
        symlink_file(managed_file, user_file)?;

        Ok(())
    }

    fn handle_only_user_file_exists(user_path: &Path, managed_file: &Path) -> Result<()> {
        fs::copy(user_path, managed_file)?;
        fs::remove_file(user_path)?;

        symlink_file(managed_file, user_path)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::AddCommand;
    use crate::config::{app_config::{AppConfig, Project}, locations::LocationsProvider};

    #[test]
    fn add_file_when_project_does_not_exist() {
        let config_dir = tempfile::tempdir().unwrap();
        let data_dir = tempfile::tempdir().unwrap();
        let current_dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(data_dir.path().join("projects/proj1")).unwrap();
        let locations_provider =
            LocationsProvider::new(config_dir.path().to_path_buf(), data_dir.path().to_path_buf());

        let user_file = current_dir.path().join("file");
        let config_file = config_dir.path().join("config.json");
        let config = AppConfig { projects: vec![] };
        fs::write(&config_file, serde_json::to_string(&config).unwrap()).unwrap();

        let sut = AddCommand::new(&locations_provider);

        let result = sut.add_file(user_file, current_dir.path(), false, false);

        assert!(result.is_err());
        let message = result.unwrap_err().to_string();
        print!("{}", message);
        assert!(message.contains("The current directory is not associated with any"));
    }

    #[test]
    fn add_file_fresh_scenario() {
        let config_dir = tempfile::tempdir().unwrap();
        let data_dir = tempfile::tempdir().unwrap();
        let current_dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(data_dir.path().join("projects/proj1")).unwrap();
        let locations_provider =
            LocationsProvider::new(config_dir.path().to_path_buf(), data_dir.path().to_path_buf());

        let user_file = current_dir.path().join("file");
        let config_file = config_dir.path().join("config.json");
        let config = AppConfig {
            projects: vec![Project { name: "proj1".into(), id: "1".into(), path: current_dir.path().to_path_buf() }],
        };
        fs::write(&config_file, serde_json::to_string(&config).unwrap()).unwrap();

        let sut = AddCommand::new(&locations_provider);

        sut.add_file(user_file, current_dir.path(), false, false).unwrap();
    }

    #[test]
    fn add_file_in_subdirectory() {
        let config_dir = tempfile::tempdir().unwrap();
        let data_dir = tempfile::tempdir().unwrap();
        let project_root = tempfile::tempdir().unwrap();
        fs::create_dir_all(data_dir.path().join("projects/proj1")).unwrap();
        let locations_provider =
            LocationsProvider::new(config_dir.path().to_path_buf(), data_dir.path().to_path_buf());

        let config_file = config_dir.path().join("config.json");
        let config = AppConfig {
            projects: vec![Project { name: "proj1".into(), id: "1".into(), path: project_root.path().to_path_buf() }],
        };
        fs::write(&config_file, serde_json::to_string(&config).unwrap()).unwrap();

        let subdir = project_root.path().join("config");
        fs::create_dir_all(&subdir).unwrap();
        let user_file = subdir.join("secrets.env");

        let sut = AddCommand::new(&locations_provider);
        sut.add_file(user_file, project_root.path(), false, false).unwrap();

        let managed_file = data_dir.path().join("projects/proj1/config/secrets.env");
        assert!(managed_file.exists());
    }

    #[test]
    fn add_file_from_subdirectory_cwd() {
        let config_dir = tempfile::tempdir().unwrap();
        let data_dir = tempfile::tempdir().unwrap();
        let project_root = tempfile::tempdir().unwrap();
        fs::create_dir_all(data_dir.path().join("projects/proj1")).unwrap();
        let locations_provider =
            LocationsProvider::new(config_dir.path().to_path_buf(), data_dir.path().to_path_buf());

        let config_file = config_dir.path().join("config.json");
        let config = AppConfig {
            projects: vec![Project { name: "proj1".into(), id: "1".into(), path: project_root.path().to_path_buf() }],
        };
        fs::write(&config_file, serde_json::to_string(&config).unwrap()).unwrap();

        let subdir = project_root.path().join("config");
        fs::create_dir_all(&subdir).unwrap();
        let user_file = subdir.join("secrets.env");

        let sut = AddCommand::new(&locations_provider);
        sut.add_file(std::path::PathBuf::from("secrets.env"), &subdir, false, false)
            .unwrap();

        let managed_file = data_dir.path().join("projects/proj1/config/secrets.env");
        assert!(managed_file.exists());

        let symlink_target = fs::read_link(&user_file).unwrap();
        assert_eq!(managed_file, symlink_target);
    }

    #[test]
    fn add_directory_fresh() {
        let config_dir = tempfile::tempdir().unwrap();
        let data_dir = tempfile::tempdir().unwrap();
        let project_root = tempfile::tempdir().unwrap();
        fs::create_dir_all(data_dir.path().join("projects/proj1")).unwrap();
        let locations_provider =
            LocationsProvider::new(config_dir.path().to_path_buf(), data_dir.path().to_path_buf());

        let config_file = config_dir.path().join("config.json");
        let config = AppConfig {
            projects: vec![Project { name: "proj1".into(), id: "1".into(), path: project_root.path().to_path_buf() }],
        };
        fs::write(&config_file, serde_json::to_string(&config).unwrap()).unwrap();

        let dir_path = project_root.path().join("secrets");
        let sut = AddCommand::new(&locations_provider);
        sut.add_file(dir_path.clone(), project_root.path(), false, true).unwrap();

        assert!(dir_path.is_symlink());
        let managed = data_dir.path().join("projects/proj1/secrets");
        assert!(managed.is_dir());
    }

    #[test]
    fn add_existing_directory() {
        let config_dir = tempfile::tempdir().unwrap();
        let data_dir = tempfile::tempdir().unwrap();
        let project_root = tempfile::tempdir().unwrap();
        fs::create_dir_all(data_dir.path().join("projects/proj1")).unwrap();
        let locations_provider =
            LocationsProvider::new(config_dir.path().to_path_buf(), data_dir.path().to_path_buf());

        let config_file = config_dir.path().join("config.json");
        let config = AppConfig {
            projects: vec![Project { name: "proj1".into(), id: "1".into(), path: project_root.path().to_path_buf() }],
        };
        fs::write(&config_file, serde_json::to_string(&config).unwrap()).unwrap();

        let dir_path = project_root.path().join("config");
        fs::create_dir_all(&dir_path).unwrap();
        fs::write(dir_path.join("db.env"), "DB_URL=postgres").unwrap();
        fs::write(dir_path.join("app.env"), "APP_KEY=secret").unwrap();

        let sut = AddCommand::new(&locations_provider);
        sut.add_file(dir_path.clone(), project_root.path(), false, false).unwrap();

        assert!(dir_path.is_symlink());
        let managed = data_dir.path().join("projects/proj1/config");
        assert!(managed.is_dir());
        assert_eq!(fs::read_to_string(managed.join("db.env")).unwrap(), "DB_URL=postgres");
        assert_eq!(fs::read_to_string(managed.join("app.env")).unwrap(), "APP_KEY=secret");
    }

    #[test]
    fn add_dir_flag_on_existing_file_fails() {
        let config_dir = tempfile::tempdir().unwrap();
        let data_dir = tempfile::tempdir().unwrap();
        let project_root = tempfile::tempdir().unwrap();
        fs::create_dir_all(data_dir.path().join("projects/proj1")).unwrap();
        let locations_provider =
            LocationsProvider::new(config_dir.path().to_path_buf(), data_dir.path().to_path_buf());

        let config_file = config_dir.path().join("config.json");
        let config = AppConfig {
            projects: vec![Project { name: "proj1".into(), id: "1".into(), path: project_root.path().to_path_buf() }],
        };
        fs::write(&config_file, serde_json::to_string(&config).unwrap()).unwrap();

        let file_path = project_root.path().join("somefile");
        fs::write(&file_path, "content").unwrap();

        let sut = AddCommand::new(&locations_provider);
        let result = sut.add_file(file_path, project_root.path(), false, true);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("--dir"));
    }

    #[test]
    fn add_file_inside_managed_dir_fails() {
        let config_dir = tempfile::tempdir().unwrap();
        let data_dir = tempfile::tempdir().unwrap();
        let project_root = tempfile::tempdir().unwrap();
        fs::create_dir_all(data_dir.path().join("projects/proj1")).unwrap();
        let locations_provider =
            LocationsProvider::new(config_dir.path().to_path_buf(), data_dir.path().to_path_buf());

        let config_file = config_dir.path().join("config.json");
        let config = AppConfig {
            projects: vec![Project { name: "proj1".into(), id: "1".into(), path: project_root.path().to_path_buf() }],
        };
        fs::write(&config_file, serde_json::to_string(&config).unwrap()).unwrap();

        // First add a directory
        let dir_path = project_root.path().join("config");
        fs::create_dir_all(&dir_path).unwrap();
        let sut = AddCommand::new(&locations_provider);
        sut.add_file(dir_path, project_root.path(), false, false).unwrap();

        // Now try to add a file inside it
        let file_inside = project_root.path().join("config/db.env");
        let result = sut.add_file(file_inside, project_root.path(), false, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already managed as part of directory"));
    }
}
