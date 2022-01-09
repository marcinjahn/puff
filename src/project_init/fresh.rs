use crate::{
    config::{app_config::{AppConfigManager}, locations},
    error::AppError,
};
use std::{
    error::Error,
    fs::{self},
    path::Path,
};

/// Initializes a fresh project that does not yet exists in conman's config
/// directory. It creates a directory to store project's secret files and 
/// updates conman's config file by adding that new project.
pub fn init_project(name: &str, user_path: &Path) -> Result<(), Box<dyn Error>> {
    let project_config_path = locations::get_project_config_path(name)?;
    if project_config_path.exists() {
        return Err(Box::new(AppError(
            "The project folder already exists in conman's configs".into(),
        )));
    }

    fs::create_dir_all(project_config_path)?;
    let manager = AppConfigManager::new()?;
    manager.add(name, user_path)?;

    Ok(())
}