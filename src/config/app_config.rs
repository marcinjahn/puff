use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    path::{Path, PathBuf}, fs::File, io::{BufReader, BufWriter},
};
use uuid::Uuid;
use crate::{config::locations, error::AppError};

#[derive(Default, Serialize, Deserialize)]
pub struct AppConfig {
    pub projects: Vec<Project>,
}

impl AppConfig {
    pub fn to_string(&self) -> Result<String, Box<dyn Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Project {
    pub name: String,
    pub id: String,
    pub path: PathBuf,
}

impl Project {

    /// Creates a new instance of Project
    pub fn new(name: &str, user_dir: &Path) -> Project {
        Project {
            name: name.to_owned(),
            path: user_dir.to_owned(),
            id: Uuid::new_v4().to_string(),
        }
    }
}

/// A manager of conman's configuration file config.json.
/// Any modifications of that file should go through this
/// struct's functions.
pub struct AppConfigManager {
    file_path: PathBuf
}

impl AppConfigManager {
    pub fn new() -> Result<AppConfigManager, Box<dyn Error>> {
        let file_path = locations::get_config_file_path()?;

        if !file_path.exists() {
            return Err(Box::new(AppError(
                "Conman's config.json file does not exist".into(),
            )));
        }

        Ok(AppConfigManager { file_path })
    }

    /// Returns the current content of the config.json file
    pub fn get_config(&self) -> Result<AppConfig, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let config: AppConfig = serde_json::from_reader(reader)?;

        Ok(config)
    }

    /// Adds a new entry to the file. If an entry with the same 'name' already
    /// exists, an error will be returned.
    /// 
    /// WARNING: The is function modifies the config.json file, even though function's
    /// signature does not have any 'mut'.
    pub fn add(&self, project_name: &str, user_dir: &Path) -> Result<(), Box<dyn Error>> {
        let mut config = self.get_config()?;
    
        if config.projects.iter().any(|p| p.name == project_name) {
            return Err(Box::new(AppError(
                format!(
                    "Conman's config.json file already contains a project named '{}'",
                    project_name
                ),
            )));
        }
    
        config.projects.push(Project::new(project_name, user_dir));
        self.save_config(&config)?;

        Ok(())
    }

    /// Saves provided config to the config.json file
    fn save_config(&self, config: &AppConfig) -> Result<(), Box<dyn Error>> {
        let file = File::create(&self.file_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &config)?;

        Ok(())
    }
}