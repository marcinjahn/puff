use super::{app_config::AppConfigManager, locations};
use crate::error::AppError;
use std::{error::Error, fs, path::Path};

pub fn is_associated(path: &Path) -> Result<bool, Box<dyn Error>> {
    let config = AppConfigManager::new()?.get_config()?;
    if config.projects.iter().any(|p| p.path == path) {
        return Ok(true);
    }

    Ok(false)
}

pub fn get_unassociated_projects() -> Result<Vec<String>, Box<dyn Error>> {
    let all = get_all_projects()?;
    let config = AppConfigManager::new()?.get_config()?;
    let associated: Vec<_> = config.projects.iter().map(|p| &p.name).collect();

    Ok(all
        .iter()
        .filter(|n| !associated.contains(n))
        .cloned()
        .collect::<Vec<_>>())
}

fn get_all_projects() -> Result<Vec<String>, Box<dyn Error>> {
    let location = locations::get_configs_config_path()?;
    let paths = fs::read_dir(location)?;

    let mut projects = vec![];
    for path in paths {
        let name = path?.file_name().into_string();
        match name {
            Ok(name) => projects.push(name),
            Err(osstr) => {
                return Err(Box::new(AppError(format!(
                    "Project's name '{:?}' could not be converted into UTF-8 string.",
                    osstr
                ))));
            }
        }
    }

    Ok(projects)
}
