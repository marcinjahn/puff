use crate::error::AppError;
use directories::ProjectDirs;
use std::{
    error::Error,
    path::{Path, PathBuf},
};

use super::app_config::AppConfigManager;

const APP_NAME: &str = "conman";

pub struct LocationsProvider {
    config_base_path: PathBuf
}

impl LocationsProvider {
    pub fn new(base_config_path: PathBuf) -> LocationsProvider {
        LocationsProvider { config_base_path: base_config_path }
    }

    // pub fn get_base_config_path() -> Result<PathBuf, Box<dyn Error>> {
    //     let dirs = get_dirs()?;
    //     let path = dirs.config_dir();
    //     Ok(path.to_path_buf())
    // }
    
    pub fn get_configs_config_path(&self) -> PathBuf {
        self.config_base_path.join(Path::new("configs"))
    }
    
    pub fn get_config_file_path(&self) -> PathBuf {
        self.config_base_path.join(Path::new("config.json"))
    }
    
    pub fn get_managed_dir(&self, name: &str) -> PathBuf {
        self.get_configs_config_path().join(Path::new(name))
    }
    
    pub fn get_project_name_by_user_dir(&self, user_dir: &Path) -> Result<String, Box<dyn Error>> {
        let config = AppConfigManager::new(self.get_config_file_path())?.get_config()?;
        
        match config.projects.iter().find(|p| p.path == user_dir) {
            None => {
                Err(Box::new(AppError(
                    "Parent directory of the provided file is not associated with any project known to conman. Did you initialize it with 'conman init'?".into(),
                )))
            },
            Some(project) => {
                Ok(project.name.clone())
            }
        }
    }
}

impl Default for LocationsProvider {
    fn default() -> Self {
        Self { config_base_path: get_base_config_path().expect("The default configuration path of conman could not be retrieved") }
    }
}

pub fn get_base_config_path() -> Result<PathBuf, Box<dyn Error>> {
    match ProjectDirs::from("com", "marcinjahn", APP_NAME) {
        Some(dirs) => Ok(dirs.config_dir().to_owned()),
        None => Err(Box::new(AppError(
            "Could not find the path of configuration files of the host.".into(),
        ))),
    }
}