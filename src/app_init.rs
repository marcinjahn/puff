use crate::config::{app_config::AppConfig, locations::LocationsProvider};
use crate::migration;
use anyhow::Result;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

/// Makes sure that config files required by puff exist on the host
pub struct AppInitializer<'a> {
    pub locations_provider: &'a LocationsProvider,
}

impl<'a> AppInitializer<'a> {
    /// Creates config dir and file if they don't exist
    pub fn init(&self) -> Result<()> {
        migration::migrate_projects_path_if_needed(self.locations_provider)?;

        let base_config_dir = self.locations_provider.get_base_config_path()?;
        if !base_config_dir.exists() {
            fs::create_dir_all(&base_config_dir)?;
        }

        let config_file_path = self.locations_provider.get_config_file_path();
        if !config_file_path.exists() {
            self.create_config_file(&config_file_path)?;
        }

        let projects_dir_path = self.locations_provider.get_projects_data_path();
        if !projects_dir_path.exists() {
            fs::create_dir_all(projects_dir_path)?;
        }

        Ok(())
    }

    fn create_config_file(&self, file_path: &Path) -> Result<()> {
        let file_content = AppConfig::default().to_string()?;
        let mut file = File::create(file_path)?;
        Ok(file.write_all(file_content.as_bytes())?)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::AppInitializer;
    use crate::config::locations::LocationsProvider;

    #[test]
    fn init_when_not_initialized_yet_files_get_created() {
        let (locations_provider, _config_path, data_path, _temp_dirs) =
            get_fake_locationsprovider(false);
        let init = AppInitializer {
            locations_provider: &locations_provider,
        };

        init.init().unwrap();
        let config_file_content =
            get_config_without_whitespace(locations_provider.get_config_file_path());

        assert_eq!("{\"projects\":[]}", config_file_content);
        assert!(data_path.join("projects").exists())
    }

    #[test]
    fn init_when_base_dir_exists_but_config_file_doesnt_then_config_file_gets_created() {
        let (locations_provider, _, _, _temp_dirs) = get_fake_locationsprovider(true);
        let init = AppInitializer {
            locations_provider: &locations_provider,
        };

        init.init().unwrap();
        let config_file_content =
            get_config_without_whitespace(locations_provider.get_config_file_path());

        assert_eq!("{\"projects\":[]}", config_file_content);
    }

    fn get_fake_locationsprovider(
        base_dir_should_exist: bool,
    ) -> (
        LocationsProvider,
        PathBuf,
        PathBuf,
        (tempfile::TempDir, tempfile::TempDir),
    ) {
        let config_dir = tempfile::tempdir().unwrap();
        let data_dir = tempfile::tempdir().unwrap();
        let mut config_path = config_dir.path().to_path_buf();
        let data_path = data_dir.path().to_path_buf();
        if !base_dir_should_exist {
            config_path = config_path.join("non-existing-base-dir");
        }
        (
            LocationsProvider::new(config_path.clone(), data_path.clone()),
            config_path,
            data_path,
            (config_dir, data_dir),
        )
    }

    fn get_config_without_whitespace(path: PathBuf) -> String {
        fs::read_to_string(path)
            .unwrap()
            .replace("\n", "")
            .replace("\r", "")
            .replace(" ", "")
    }
}
