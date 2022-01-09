use std::{path::Path, error::Error};

use crate::{config::{locations, app_config::{AppConfigManager}}, error::AppError};

/// Initializes a project that already exists in conman's configs
/// directory. It updates conman's config file by adding that new project there.
pub fn init_project(name: &str, path: &Path) -> Result<(), Box<dyn Error>> {
    let project_config_path = locations::get_project_config_path(name)?;
    if !project_config_path.exists() {
        return Err(Box::new(AppError(
            "The project folder does not exist in conman's configs".into(),
        )));
    }

    let manager = AppConfigManager::new()?;
    manager.add(name, path)?;

    // TODO: Bring in existing project's secrets

    Ok(())
}