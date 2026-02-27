use super::{app_config::AppConfig, locations::LocationsProvider};
use anyhow::{Result, bail};
use std::{
    fs,
    path::{Path, PathBuf},
};

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

    pub fn is_associated(&self, path: &Path) -> Result<bool> {
        Ok(self.app_config.projects.iter().any(|p| p.path == path))
    }

    pub fn get_details(&self, project_name: &str) -> Result<Option<ProjectDetails>> {
        let project_config = self
            .app_config
            .projects
            .iter()
            .find(|p| p.name == project_name);

        let managed_dir = self.locations_provider.get_managed_dir(project_name);
        if !managed_dir.exists() {
            return Ok(None);
        }

        let files = collect_files_recursively(&managed_dir, &managed_dir)?;
        let info = ProjectInfo {
            name: project_name.to_owned(),
            managed_dir,
            files,
        };

        Ok(Some(match project_config {
            Some(config) => ProjectDetails::Associated(AssociatedProject {
                info,
                user_dir: config.path.clone(),
            }),
            None => ProjectDetails::Unassociated(info),
        }))
    }

    pub fn get_associated_projects(&self) -> Vec<String> {
        self.app_config
            .projects
            .iter()
            .map(|p| p.name.clone())
            .collect::<Vec<String>>()
    }

    /// Returns projects' names that exist in puff, but have not yet been associated
    /// with any user's directory
    pub fn get_unassociated_projects(&self) -> Result<Vec<String>> {
        let all = self.get_all_projects()?;
        let associated: Vec<_> = self.app_config.projects.iter().map(|p| &p.name).collect();

        if all.len() < associated.len() {
            // TODO: What should user do in such scenario?
            bail!(
                "puff configuration is corrupted: the registry references projects that no longer exist on disk."
            );
        }

        Ok(all
            .iter()
            .filter(|n| !associated.contains(n))
            .cloned()
            .collect::<Vec<_>>())
    }

    /// Returns names of all the projects that puff stores (some of them might
    /// not be associated yet)
    fn get_all_projects(&self) -> Result<Vec<String>> {
        let location = self.locations_provider.get_configs_config_path();
        let paths = fs::read_dir(location)?;

        let mut projects = vec![];
        for path in paths {
            let name = path?.file_name().into_string();
            match name {
                Ok(name) => projects.push(name),
                Err(osstr) => {
                    bail!("Project name '{:?}' is not valid UTF-8.", osstr);
                }
            }
        }

        Ok(projects)
    }
}

/// Fields common to all puff-managed projects regardless of association status.
#[non_exhaustive]
pub struct ProjectInfo {
    pub name: String,
    pub managed_dir: PathBuf,
    pub files: Vec<PathBuf>,
}

#[non_exhaustive]
pub struct AssociatedProject {
    pub info: ProjectInfo,
    pub user_dir: PathBuf,
}

/// Details of a puff-managed project.
#[non_exhaustive]
pub enum ProjectDetails {
    /// Project is "connected" via symlinks to actual files on user's machine
    Associated(AssociatedProject),

    /// Project is available in puff, probably was migrated from another machine, but it is not yet
    /// connected to any directory on user's machine.
    Unassociated(ProjectInfo),
}

impl ProjectDetails {
    pub fn info(&self) -> &ProjectInfo {
        match self {
            ProjectDetails::Associated(associated) => &associated.info,
            ProjectDetails::Unassociated(info) => info,
        }
    }
}

/// Collects all files under `dir`, returning their paths relative to `base`.
fn collect_files_recursively(base: &Path, dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = vec![];
    let mut dirs = vec![dir.to_owned()];
    while let Some(current) = dirs.pop() {
        for entry in fs::read_dir(&current)? {
            let path = entry?.path();
            if path.is_dir() {
                dirs.push(path);
            } else {
                files.push(path.strip_prefix(base)?.to_owned());
            }
        }
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use crate::config::app_config::{AppConfig, Project};
    use crate::config::locations::LocationsProvider;
    use std::fs::{self};

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

    // TODO: Test get_associated_projects fn
}
