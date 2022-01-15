use crate::{
    config::{app_config::AppConfigManager},
    error::AppError,
    fs_utils::{backup_file},
};
use std::{error::Error, fs, path::Path};


/// Initializes a project that already exists in conman's configs
/// directory.
pub struct ExistingProjectInitializer<'a> {
    app_config_manager: &'a AppConfigManager
}

impl<'a> ExistingProjectInitializer<'a> {
    pub fn new(app_config_manager: &'a AppConfigManager) -> Self {
        ExistingProjectInitializer { app_config_manager }
    }

    /// It updates conman's config file by adding that new project there.
    pub fn init_project(&self, name: &str, user_dir: &Path, managed_dir: &Path) -> Result<(), Box<dyn Error>> {
        if !managed_dir.exists() {
            return Err(Box::new(AppError(
                "The project folder does not exist in conman's configs".into(),
            )));
        }

        self.app_config_manager.add(name, user_dir)?;

        self.bring_in_existing_secrets(name, user_dir, managed_dir)?;

        Ok(())
    }

    /// Initializes the user's project directory with files managed by conman
    fn bring_in_existing_secrets(&self, _project_name: &str, user_dir: &Path, managed_dir: &Path) -> Result<(), Box<dyn Error>> {
        for file in managed_dir.read_dir()? {
            match file {
                Ok(file) => {
                    self.handle_existing_file(&file.path(), user_dir)?;
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
    fn handle_existing_file(&self, managed_file: &Path, user_dir: &Path) -> Result<(), Box<dyn Error>> {
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

        symlink::symlink_file(managed_file, file_in_user_dir)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{File, self};
    use std::io::{Write, BufReader};
    use std::path::{PathBuf, Path};
    use crate::config::app_config::{AppConfigManager, AppConfig};
    use super::ExistingProjectInitializer;

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

        sut.init_project(project_name, user_dir.path(), managed_dir.path()).unwrap();

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

        sut.init_project(project_name, user_dir.path(), managed_dir.path()).unwrap();
        
        let mut symlinks = user_dir.path()
            .read_dir().unwrap()
            .map(|f| String::from(f.unwrap().path().to_str().unwrap()))
            .collect::<Vec<String>>();
        symlinks.sort();

        assert_eq!(2, symlinks.len());

        for (index, file) in symlinks.iter().enumerate() {
            if let Ok(symlink_path) = fs::read_link(file) {
                let index = index + 1;
                assert_eq!(managed_dir.path().join(format!("file{index}")), symlink_path);
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