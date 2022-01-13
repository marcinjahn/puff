use super::{app_config::AppConfig, locations::LocationsProvider};
use crate::error::AppError;
use std::{error::Error, fs, path::Path};

pub struct ProjectsRetriever<'a> {
    pub app_config: AppConfig,
    pub locations_provider: &'a LocationsProvider,
}

impl<'a> ProjectsRetriever<'a> {
    pub fn is_associated(&self, path: &Path) -> Result<bool, Box<dyn Error>> {
        if self.app_config.projects.iter().any(|p| p.path == path) {
            return Ok(true);
        }

        Ok(false)
    }

    pub fn get_unassociated_projects(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let all = self.get_all_projects()?;
        let associated: Vec<_> = self.app_config.projects.iter().map(|p| &p.name).collect();

        Ok(all
            .iter()
            .filter(|n| !associated.contains(n))
            .cloned()
            .collect::<Vec<_>>())
    }

    fn get_all_projects(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let location = self.locations_provider.get_configs_config_path();
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
}
