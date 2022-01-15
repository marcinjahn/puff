use crate::config::{app_config::AppConfig, locations::LocationsProvider};
use std::{
    error::Error,
    fs::{self, File},
    io::Write,
    path::Path,
};

/// Makes sure that config files required by conman exist on the host
pub struct AppInitializer<'a> {
    pub locations_provider: &'a LocationsProvider,
}

impl<'a> AppInitializer<'a> {
    /// Creates config dir and file if they don't exist
    pub fn init(&self) -> Result<(), Box<dyn Error>> {
        let base_config_dir = self.locations_provider.get_base_config_path()?;
        if !base_config_dir.exists() {
            let base_config_dir = self.locations_provider.get_configs_config_path();
            fs::create_dir_all(base_config_dir)?;
        }

        let config_file_path = self.locations_provider.get_config_file_path();
        if !config_file_path.exists() {
            self.create_config_file(&config_file_path)?;
        }

        let configs_dir_path = self.locations_provider.get_configs_config_path();
        if !configs_dir_path.exists() {
            fs::create_dir_all(configs_dir_path)?;
        }

        Ok(())
    }

    fn create_config_file(&self, file_path: &Path) -> Result<(), Box<dyn Error>> {
        let file_content = AppConfig::default().to_string()?;

        let mut file = match File::create(&file_path) {
            Err(err) => {
                return Err(Box::new(err));
            }
            Ok(file) => file,
        };

        Ok(file.write_all(file_content.as_bytes())?)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::test_utils::config::get_config_without_whitespace;
    use crate::config::locations::LocationsProvider;
    use super::AppInitializer;

    #[test]
    fn init_when_not_initialized_yet_files_get_created() {
        let (locations_provider, base_path, _temp_dir) = get_fake_locationsprovider(false);
        let init = AppInitializer {
            locations_provider: &locations_provider,
        };

        init.init().unwrap();
        let config_file_content =
            get_config_without_whitespace(locations_provider.get_config_file_path());

        assert_eq!("{\"projects\":[]}", config_file_content);
        assert!(base_path.join("configs").exists())
    }

    #[test]
    fn init_when_base_dir_exists_but_config_file_doesnt_then_config_file_gets_created() {
        let (locations_provider, _, _temp_dir) = get_fake_locationsprovider(true);
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
    ) -> (LocationsProvider, PathBuf, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let mut path = dir.path().to_path_buf();
        if !base_dir_should_exist {
            path = path.join("non-existing-base-dir");
        }
        (LocationsProvider::new(path.to_path_buf()), path, dir)
    }
}
