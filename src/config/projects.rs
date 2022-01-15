use super::{app_config::AppConfig, locations::LocationsProvider};
use crate::error::AppError;
use std::{error::Error, fs, path::Path};

pub struct ProjectsRetriever<'a> {
    app_config: AppConfig,
    locations_provider: &'a LocationsProvider,
}

impl<'a> ProjectsRetriever<'a> {
    pub fn new(app_config: AppConfig, locations_provider: &'a LocationsProvider) -> Self {
        ProjectsRetriever {
            app_config,
            locations_provider,
        }
    }

    pub fn is_associated(&self, path: &Path) -> Result<bool, Box<dyn Error>> {
        if self.app_config.projects.iter().any(|p| p.path == path) {
            return Ok(true);
        }

        Ok(false)
    }

    /// Returns projects' names that exist in conman, but have not yet been associated
    /// with any user's directory
    pub fn get_unassociated_projects(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let all = self.get_all_projects()?;
        let associated: Vec<_> = self.app_config.projects.iter().map(|p| &p.name).collect();

        if all.len() < associated.len() {
            // TODO: What should user do in such scenario?
            return Err(Box::new(AppError(format!(
                "conman's config file is corrupted. It contains projects that do not exist."
            ))));
        }

        Ok(all
            .iter()
            .filter(|n| !associated.contains(n))
            .cloned()
            .collect::<Vec<_>>())
    }

    /// Returns names of all the projects that conman stores (some of them might
    /// not be associated yet)
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

#[cfg(test)]
mod tests {
    use crate::config::app_config::{AppConfig, AppConfigManager, Project};
    use crate::config::locations::LocationsProvider;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;

    use super::ProjectsRetriever;

    #[test]
    fn is_associated_when_associated_project_is_provided_true_is_returned() {
        let checked_dir = tempfile::tempdir().unwrap();
        let base_dir = tempfile::tempdir().unwrap();
        let locations_provider = LocationsProvider::new(base_dir.path().to_path_buf());
        let app_config = AppConfig {
            projects: vec![Project {
                name: String::from("proj"),
                id: String::from("1"),
                path: Path::new(checked_dir.path().to_str().unwrap()).to_path_buf(),
            }],
        };

        let sut = ProjectsRetriever::new(app_config, &locations_provider);

        let result = sut.is_associated(checked_dir.path()).unwrap();

        assert!(result);
    }

    #[test]
    fn is_associated_when_not_associated_project_is_provided_false_is_returned() {
        let base_dir = tempfile::tempdir().unwrap();
        let locations_provider = LocationsProvider::new(base_dir.path().to_path_buf());
        let app_config = AppConfig { projects: vec![] };

        let sut = ProjectsRetriever::new(app_config, &locations_provider);

        let some_dir = tempfile::tempdir().unwrap();
        let result = sut.is_associated(some_dir.path()).unwrap();

        assert!(!result);
    }

    #[test]
    fn get_unassociated_projects_when_all_projects_are_associated_empty_vector_is_returned() {
        let proj_1_dir = tempfile::tempdir().unwrap();
        let proj_2_dir = tempfile::tempdir().unwrap();
        let proj_3_dir = tempfile::tempdir().unwrap();

        let base_dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(base_dir.path().join("configs/proj1")).unwrap();
        fs::create_dir_all(base_dir.path().join("configs/proj2")).unwrap();
        fs::create_dir_all(base_dir.path().join("configs/proj3")).unwrap();

        let locations_provider = LocationsProvider::new(base_dir.path().to_path_buf());
        let app_config = AppConfig {
            projects: vec![
                Project {
                    name: String::from("proj1"),
                    id: String::from("1"),
                    path: Path::new(proj_1_dir.path().to_str().unwrap()).to_path_buf(),
                },
                Project {
                    name: String::from("proj2"),
                    id: String::from("2"),
                    path: Path::new(proj_2_dir.path().to_str().unwrap()).to_path_buf(),
                },
                Project {
                    name: String::from("proj3"),
                    id: String::from("3"),
                    path: Path::new(proj_3_dir.path().to_str().unwrap()).to_path_buf(),
                },
            ],
        };

        let sut = ProjectsRetriever::new(app_config, &locations_provider);

        let result = sut.get_unassociated_projects().unwrap();

        assert!(result.len() == 0);
    }

    #[test]
    fn get_unassociated_projects_when_some_projects_are_associated_proper_vector_is_returned() {
        let proj_1_dir = tempfile::tempdir().unwrap();
        let proj_2_dir = tempfile::tempdir().unwrap();

        let base_dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(base_dir.path().join("configs/proj1")).unwrap();
        fs::create_dir_all(base_dir.path().join("configs/proj2")).unwrap();
        fs::create_dir_all(base_dir.path().join("configs/proj3")).unwrap();

        let locations_provider = LocationsProvider::new(base_dir.path().to_path_buf());
        let app_config = AppConfig {
            projects: vec![
                Project {
                    name: String::from("proj1"),
                    id: String::from("1"),
                    path: Path::new(proj_1_dir.path().to_str().unwrap()).to_path_buf(),
                },
                Project {
                    name: String::from("proj2"),
                    id: String::from("2"),
                    path: Path::new(proj_2_dir.path().to_str().unwrap()).to_path_buf(),
                },
            ],
        };

        let sut = ProjectsRetriever::new(app_config, &locations_provider);

        let result = sut.get_unassociated_projects().unwrap();

        assert_eq!(1, result.len());
        assert!(result.first().unwrap() == "proj3");
    }

    #[test]
    fn get_all_projects_when_some_projects_exist_then_proper_vector_is_returned() {
        let proj_1_dir = tempfile::tempdir().unwrap();
        let proj_2_dir = tempfile::tempdir().unwrap();

        let base_dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(base_dir.path().join("configs/proj1")).unwrap();
        fs::create_dir_all(base_dir.path().join("configs/proj2")).unwrap();
        fs::create_dir_all(base_dir.path().join("configs/proj3")).unwrap();

        let locations_provider = LocationsProvider::new(base_dir.path().to_path_buf());
        let app_config = AppConfig {
            projects: vec![
                Project {
                    name: String::from("proj1"),
                    id: String::from("1"),
                    path: Path::new(proj_1_dir.path().to_str().unwrap()).to_path_buf(),
                },
                Project {
                    name: String::from("proj2"),
                    id: String::from("2"),
                    path: Path::new(proj_2_dir.path().to_str().unwrap()).to_path_buf(),
                },
            ],
        };

        let sut = ProjectsRetriever::new(app_config, &locations_provider);

        let mut result = sut.get_all_projects().unwrap();
        result.sort();

        assert_eq!(3, result.len());
        assert_eq!("proj1", result[0]);
        assert_eq!("proj2", result[1]);
        assert_eq!("proj3", result[2]);
    }

    #[test]
    fn get_all_projects_when_there_are_no_projects_then_empty_vector_is_returned() {
        let base_dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(base_dir.path().join("configs")).unwrap();

        let locations_provider = LocationsProvider::new(base_dir.path().to_path_buf());
        let app_config = AppConfig { projects: vec![] };

        let sut = ProjectsRetriever::new(app_config, &locations_provider);

        let mut result = sut.get_all_projects().unwrap();
        result.sort();

        assert_eq!(0, result.len());
    }
}
