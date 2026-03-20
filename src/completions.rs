use clap_complete::engine::CompletionCandidate;

use crate::config::app_config::AppConfigManager;
use crate::config::locations::LocationsProvider;
use crate::config::projects::ProjectsRetriever;

pub fn complete_project_name(current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    let Some(current) = current.to_str() else {
        return vec![];
    };

    let locations = LocationsProvider::default();
    let Ok(config_manager) = AppConfigManager::new(locations.get_config_file_path()) else {
        return vec![];
    };
    let Ok(config) = config_manager.get_config() else {
        return vec![];
    };

    config
        .projects
        .into_iter()
        .map(|p| p.name)
        .filter(|name| name.starts_with(current))
        .map(CompletionCandidate::new)
        .collect()
}

pub fn complete_unassociated_project_name(current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    let Some(current) = current.to_str() else {
        return vec![];
    };

    let locations = LocationsProvider::default();
    let Ok(config_manager) = AppConfigManager::new(locations.get_config_file_path()) else {
        return vec![];
    };
    let Ok(config) = config_manager.get_config() else {
        return vec![];
    };

    let retriever = ProjectsRetriever::new(config, &locations);
    let Ok(unassociated) = retriever.get_unassociated_projects() else {
        return vec![];
    };

    unassociated
        .into_iter()
        .filter(|name| name.starts_with(current))
        .map(CompletionCandidate::new)
        .collect()
}
