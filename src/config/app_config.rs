use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};
use uuid::Uuid;

#[derive(Default, Serialize, Deserialize, Clone)]
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

/// A manager of puff's configuration file config.json.
/// Any modifications of that file should go through this
/// struct's functions.
pub struct AppConfigManager {
    pub config_file_path: PathBuf,
}

impl AppConfigManager {
    pub fn new(config_file_path: PathBuf) -> Result<AppConfigManager, Box<dyn Error>> {
        if !config_file_path.exists() {
            return Err(Box::new(AppError(
                "puff's config.json file does not exist".into(),
            )));
        }

        Ok(AppConfigManager { config_file_path })
    }

    /// Returns the current content of the config.json file
    pub fn get_config(&self) -> Result<AppConfig, Box<dyn Error>> {
        let file = File::open(&self.config_file_path)?;
        let reader = BufReader::new(file);
        let config: AppConfig = serde_json::from_reader(reader)?;

        Ok(config)
    }

    /// Adds a new entry to the file. If an entry with the same 'name' already
    /// exists, an error will be returned.
    ///
    /// WARNING: The is function modifies the config.json file, even though function's
    /// signature does not have any 'mut'.
    pub fn add_project(&self, project_name: &str, user_dir: &Path) -> Result<(), Box<dyn Error>> {
        let mut config = self.get_config()?;

        if config.projects.iter().any(|p| p.name == project_name) {
            return Err(Box::new(AppError(format!(
                "puff's config.json file already contains a project named '{}'",
                project_name
            ))));
        }

        config.projects.push(Project::new(project_name, user_dir));
        self.save_config(&config)?;

        Ok(())
    }

    /// Saves provided config to the config.json file
    fn save_config(&self, config: &AppConfig) -> Result<(), Box<dyn Error>> {
        let file = File::create(&self.config_file_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &config)?;

        Ok(())
    }

    pub(crate) fn remove_project(&self, name: &str) -> Result<(), Box<dyn Error>> {
        let mut config = self.get_config()?;

        let index = config.projects.iter().position(|p| p.name == name);
        if index.is_none() {
            return Ok(());
        }

        config.projects.remove(index.unwrap());
        self.save_config(&config)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, path::Path};
    use std::io::Write;

    use super::{AppConfigManager, Project, AppConfig};

    #[test]
    fn get_config_there_are_zero_projects_proper_config_gets_returned() {
        let base_dir = tempfile::tempdir().unwrap();
        let config_file = base_dir.path().join("config.json");
        let mut file = File::create(&config_file).unwrap();
        write!(file, "{{\"projects\":[]}}").unwrap();
        let config_manager = AppConfigManager::new(config_file.clone()).unwrap();

        let config = config_manager.get_config().unwrap();

        assert_eq!(0, config.projects.len());
    }

    #[test]
    fn get_config_there_are_some_projects_proper_config_gets_returned() {
        let base_dir = tempfile::tempdir().unwrap();
        let config_file = base_dir.path().join("config.json");
        let mut file = File::create(&config_file).unwrap();
        write!(file, "{{\"projects\":[{{\"name\":\"name1\", \"path\":\"path1\", \"id\":\"1\"}},{{\"name\":\"name2\", \"path\":\"path2\", \"id\":\"1\"}}]}}").unwrap();
        let config_manager = AppConfigManager::new(config_file.clone()).unwrap();

        let config = config_manager.get_config().unwrap();

        assert_eq!(2, config.projects.len());
    }

    #[test]
    fn add_project_project_gets_added_to_file() {
        let base_dir = tempfile::tempdir().unwrap();
        let config_file = base_dir.path().join("config.json");
        let mut file = File::create(&config_file).unwrap();
        write!(
            file,
            "{{\"projects\":[{{\"name\":\"name1\", \"path\":\"path1\", \"id\":\"1\"}}]}}"
        )
        .unwrap();
        let config_manager = AppConfigManager::new(config_file.clone()).unwrap();

        let new_proj_dir = tempfile::tempdir().unwrap();
        config_manager.add_project("new_proj", new_proj_dir.path()).unwrap();

        let file = config_manager.get_config().unwrap();

        assert_eq!(2, file.projects.len());
        assert!(file.projects.iter().any(|p| p.name == "new_proj" && p.path.to_str() == new_proj_dir.path().to_str()));
    }

    #[test]
    fn save_config_config_gets_saved() {
        let base_dir = tempfile::tempdir().unwrap();
        let config_file = base_dir.path().join("config.json");
        let mut file = File::create(&config_file).unwrap();
        write!(
            file,
            "{{\"projects\":[{{\"name\":\"name1\", \"path\":\"path1\", \"id\":\"1\"}}]}}"
        )
        .unwrap();
        let config_manager = AppConfigManager::new(config_file.clone()).unwrap();

        let app_config = AppConfig {
            projects: vec![
                Project {
                    name: String::from("proj1"),
                    id: String::from("1"),
                    path: Path::new(base_dir.path().to_str().unwrap()).to_path_buf(),
                },
                Project {
                    name: String::from("proj2"),
                    id: String::from("2"),
                    path: Path::new(base_dir.path().to_str().unwrap()).to_path_buf(),
                },
            ],
        };

        config_manager.save_config(&app_config).unwrap();

        let retrieved_config = config_manager.get_config().unwrap();

        assert_eq!(2, retrieved_config.projects.len());
    }
}
