use crate::{
    config::app_config::AppConfigManager,
    fs_utils::{backup_dir, backup_file, symlink_dir, symlink_file},
    managed_dirs,
};
use anyhow::{Result, anyhow, bail};
use std::{fs, path::Path};

/// Initializes a project that already exists in puff's configs
/// directory.
pub struct ExistingProjectInitializer<'a> {
    app_config_manager: &'a AppConfigManager,
}

impl<'a> ExistingProjectInitializer<'a> {
    pub fn new(app_config_manager: &'a AppConfigManager) -> Self {
        ExistingProjectInitializer { app_config_manager }
    }

    /// It updates puff's config file by adding that new project there.
    pub fn init_project(&self, name: &str, user_dir: &Path, managed_dir: &Path) -> Result<()> {
        if !managed_dir.exists() {
            bail!("The project folder does not exist in puff's configs");
        }

        self.app_config_manager.add_project(name, user_dir)?;

        create_symlinks_for_managed_files(user_dir, managed_dir)?;

        Ok(())
    }
}

/// Creates symlinks in `target_dir` for all files in `managed_dir`,
/// preserving directory structure.
pub fn create_symlinks_for_managed_files(target_dir: &Path, managed_dir: &Path) -> Result<()> {
    walk_managed_dir(target_dir, managed_dir, managed_dir)
}

fn walk_managed_dir(target_dir: &Path, managed_dir: &Path, current_dir: &Path) -> Result<()> {
    let managed_dir_set = managed_dirs::read_managed_dirs_set(managed_dir)?;
    let managed_dirs_filename = managed_dirs::managed_dirs_filename();

    walk_managed_dir_inner(
        target_dir,
        managed_dir,
        current_dir,
        &managed_dir_set,
        managed_dirs_filename,
    )
}

fn walk_managed_dir_inner(
    target_dir: &Path,
    managed_dir: &Path,
    current_dir: &Path,
    managed_dir_set: &std::collections::HashSet<std::path::PathBuf>,
    managed_dirs_filename: &str,
) -> Result<()> {
    for entry in current_dir.read_dir()? {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                let relative_path = path.strip_prefix(managed_dir)?;

                if path.is_file()
                    && path
                        .file_name()
                        .map(|n| n == managed_dirs_filename)
                        .unwrap_or(false)
                    && path.parent() == Some(managed_dir)
                {
                    continue;
                }

                if path.is_dir() {
                    if managed_dir_set.contains(&relative_path.to_path_buf()) {
                        symlink_one_dir(&path, target_dir, relative_path)?;
                    } else {
                        walk_managed_dir_inner(
                            target_dir,
                            managed_dir,
                            &path,
                            managed_dir_set,
                            managed_dirs_filename,
                        )?;
                    }
                } else {
                    symlink_one_file(&path, target_dir, relative_path)?;
                }
            }
            Err(_err) => {
                bail!(
                    "The project already contains some files, but some of them could not be read"
                );
            }
        }
    }
    Ok(())
}

fn symlink_one_dir(managed_path: &Path, target_dir: &Path, relative_path: &Path) -> Result<()> {
    let dir_in_target = target_dir.join(relative_path);
    fs::create_dir_all(dir_in_target.parent().unwrap())?;

    if let Ok(target) = fs::read_link(&dir_in_target)
        && target == managed_path
    {
        return Ok(());
    }

    if dir_in_target.exists() || dir_in_target.symlink_metadata().is_ok() {
        if dir_in_target.is_dir() && !dir_in_target.is_symlink() {
            let backup = backup_dir(&dir_in_target)?;
            fs::remove_dir_all(&dir_in_target)?;
            println!(
                "Conflict: {:?} exists as a real directory. \
                A backup was created at {}. \
                It now points to the puff-managed version.",
                relative_path
                    .file_name()
                    .unwrap_or(relative_path.as_os_str()),
                backup.unwrap(),
            );
        } else {
            let backup = backup_file(&dir_in_target)?;
            fs::remove_file(&dir_in_target)?;
            println!(
                "Conflict: {:?} exists in both the project directory and puff's registry. \
                A backup of the original was created at {}.",
                relative_path
                    .file_name()
                    .unwrap_or(relative_path.as_os_str()),
                backup.unwrap(),
            );
        }
    }

    symlink_dir(managed_path, &dir_in_target)?;
    Ok(())
}

fn symlink_one_file(managed_file: &Path, target_dir: &Path, relative_path: &Path) -> Result<()> {
    let managed_file_name = managed_file
        .file_name()
        .ok_or_else(|| anyhow!("Existing file {:?} could not be read", managed_file))?;

    let file_in_target_dir = target_dir.join(relative_path);
    fs::create_dir_all(file_in_target_dir.parent().unwrap())?;

    if let Ok(target) = fs::read_link(&file_in_target_dir)
        && target == managed_file
    {
        return Ok(());
    }

    if file_in_target_dir.exists() || file_in_target_dir.symlink_metadata().is_ok() {
        let backup = backup_file(&file_in_target_dir)?;
        fs::remove_file(&file_in_target_dir)?;
        println!(
            "Conflict: {:?} exists in both the project directory and puff's registry. \
            A backup of the original file was created at {}. \
            {:?} now points to the puff-managed version. \
            Review the backup before committing.",
            managed_file_name,
            backup.unwrap(),
            file_in_target_dir.file_name().unwrap()
        );
    }

    symlink_file(managed_file, &file_in_target_dir)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::ExistingProjectInitializer;
    use crate::config::app_config::{AppConfig, AppConfigManager};
    use std::fs::{self, File};
    use std::io::{BufReader, Write};
    use std::path::Path;

    #[test]
    fn init_project_gets_added_to_the_config_file() {
        let base_dir = tempfile::tempdir().unwrap();
        let config_file = base_dir.path().join("config.json");
        let mut file = File::create(&config_file).unwrap();
        write!(file, "{{\"projects\":[]}}").unwrap();
        // let (config_file, _) = prepare_sut_and_stuff();
        let config_manager = AppConfigManager::new(config_file.clone()).unwrap();
        let sut = ExistingProjectInitializer::new(&config_manager);

        let project_name = "some-project";
        let user_dir = tempfile::tempdir().unwrap();
        let managed_dir = tempfile::tempdir().unwrap();

        sut.init_project(project_name, user_dir.path(), managed_dir.path())
            .unwrap();

        let config_file = File::open(config_file).unwrap();
        let reader = BufReader::new(config_file);
        let config_file: AppConfig = serde_json::from_reader(reader).unwrap();

        assert_eq!(project_name, config_file.projects.first().unwrap().name);
        assert_eq!(user_dir.path(), config_file.projects.first().unwrap().path);
    }

    #[test]
    fn init_existing_secrets_get_symlinked_in_user_dir() {
        let base_dir = tempfile::tempdir().unwrap();
        let config_file = base_dir.path().join("config.json");
        let mut file = File::create(&config_file).unwrap();
        write!(file, "{{\"projects\":[]}}").unwrap();
        // let (config_file, _) = prepare_sut_and_stuff();
        let config_manager = AppConfigManager::new(config_file.clone()).unwrap();
        let sut = ExistingProjectInitializer::new(&config_manager);

        let project_name = "some-project";
        let user_dir = tempfile::tempdir().unwrap();
        let managed_dir = tempfile::tempdir().unwrap();

        create_file(&managed_dir.path().join("file1"), "abc");
        create_file(&managed_dir.path().join("file2"), "def");

        sut.init_project(project_name, user_dir.path(), managed_dir.path())
            .unwrap();

        let mut symlinks = user_dir
            .path()
            .read_dir()
            .unwrap()
            .map(|f| String::from(f.unwrap().path().to_str().unwrap()))
            .collect::<Vec<String>>();
        symlinks.sort();

        assert_eq!(2, symlinks.len());

        for (index, file) in symlinks.iter().enumerate() {
            if let Ok(symlink_path) = fs::read_link(file) {
                let index = index + 1;
                assert_eq!(
                    managed_dir.path().join(format!("file{index}")),
                    symlink_path
                );
            } else {
                panic!("File is not a soft link")
            }
        }
    }

    fn create_file(path: &Path, content: &str) {
        let mut file = File::create(&path).unwrap();
        write!(file, "{content}").unwrap();
    }

    // fn prepare_sut_and_stuff<'a>() -> (ExistingProjectInitializer<'a>, AppConfigManager, PathBuf, tempfile::TempDir) {
    //     let base_dir = tempfile::tempdir().unwrap();
    //     let file_path = base_dir.path().join("config.json");
    //     let mut file = File::create(&file_path).unwrap();
    //     write!(file, "{{\"projects\":[]}}").unwrap();

    //     let config_manager = Rc::new(AppConfigManager::new(file_path).unwrap());

    //     let sut = ExistingProjectInitializer::new(config_manager.);

    //     (sut, config_manager, file_path, base_dir)
    // }
}
