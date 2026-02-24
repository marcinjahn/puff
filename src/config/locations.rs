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

    /// Walks up from `path` through its ancestors and returns the first
    /// (deepest/most-specific) registered project root found, along with the
    /// project name. Mirrors how git handles nested repos.
    pub fn find_project_for_path(&self, path: &Path) -> Result<(String, PathBuf)> {
        let config = AppConfigManager::new(self.get_config_file_path())?.get_config()?;

        for ancestor in path.ancestors() {
            if let Some(project) = config.projects.iter().find(|p| p.path == ancestor) {
                return Ok((project.name.clone(), project.path.clone()));
            }
        }

        Err(anyhow!(
            "The current directory is not associated with any puff project. Run 'puff init' to initialize it."
        ))
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
