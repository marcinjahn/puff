use crate::error::AppError;
use directories::ProjectDirs;
use std::{
    error::Error,
    path::{Path, PathBuf},
};

const APP_NAME: &str = "conman";

pub fn get_base_config_path() -> Result<PathBuf, Box<dyn Error>> {
    let dirs = get_dirs()?;
    let path = dirs.config_dir();
    Ok(path.to_path_buf())
}

pub fn get_configs_config_path() -> Result<PathBuf, Box<dyn Error>> {
    let path = get_base_config_path()?;
    let path = path.join(Path::new("configs"));
    Ok(path)
}

pub fn get_config_file_path() -> Result<PathBuf, Box<dyn Error>> {
    let path = get_base_config_path()?;
    let path = path.join(Path::new("config.json"));
    Ok(path)
}

pub fn get_project_config_path(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let path = get_configs_config_path()?;
    let path = path.join(Path::new(name));
    Ok(path)
}

fn get_dirs() -> Result<ProjectDirs, Box<dyn Error>> {
    match ProjectDirs::from("com", "marcinjahn", APP_NAME) {
        Some(dirs) => Ok(dirs),
        None => Err(Box::new(AppError(
            "Could not find the path of configuration files of the host.".into(),
        ))),
    }
}
