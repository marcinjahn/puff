use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use std::path::{Path, PathBuf};

use super::app_config::AppConfigManager;

const APP_NAME: &str = "puff";

pub struct LocationsProvider {
    config_base_path: PathBuf,
}

impl LocationsProvider {
    pub fn new(base_config_path: PathBuf) -> LocationsProvider {
        LocationsProvider { config_base_path: base_config_path }
    }

    pub fn get_base_config_path(&self) -> Result<PathBuf> {
        Ok(self.config_base_path.clone())
    }

    pub fn get_configs_config_path(&self) -> PathBuf {
        self.config_base_path.join(Path::new("configs"))
    }

    pub fn get_config_file_path(&self) -> PathBuf {
        self.config_base_path.join(Path::new("config.json"))
    }

    pub fn get_managed_dir(&self, name: &str) -> PathBuf {
        self.get_configs_config_path().join(Path::new(name))
    }

    pub fn get_project_name_by_user_dir(&self, user_dir: &Path) -> Result<String> {
        let config = AppConfigManager::new(self.get_config_file_path())?.get_config()?;

        match config.projects.iter().find(|p| p.path == user_dir) {
            None => Err(anyhow!(
                "The current directory is not associated with any puff project. Run 'puff init' to initialize it."
            )),
            Some(project) => Ok(project.name.clone()),
        }
    }
}

impl Default for LocationsProvider {
    fn default() -> Self {
        Self {
            config_base_path: get_base_config_path()
                .expect("The default configuration path of puff could not be retrieved"),
        }
    }
}

fn get_base_config_path() -> Result<PathBuf> {
    match ProjectDirs::from("com", "marcinjahn", APP_NAME) {
        Some(dirs) => Ok(dirs.config_dir().to_owned()),
        None => Err(anyhow!(
            "Could not determine the configuration directory for this system."
        )),
    }
}
