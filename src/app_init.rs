use crate::config::locations;
use crate::config::{app_config::AppConfig, locations::LocationsProvider};
use std::{
    error::Error,
    fs::{self, File},
    io::Write,
};

pub struct AppInitializer<'a> {
    pub locations_provider: &'a LocationsProvider,
}

impl<'a> AppInitializer<'a> {
    pub fn init(&self) -> Result<(), Box<dyn Error>> {
        let config_path = locations::get_base_config_path()?;
        if !config_path.exists() {
            self.create_app_files()?
        }

        Ok(())
    }

    fn create_app_files(&self) -> Result<(), Box<dyn Error>> {
        let configs_path = self.locations_provider.get_configs_config_path();
        fs::create_dir_all(configs_path)?;
        self.create_config_file()?;

        Ok(())
    }

    fn create_config_file(&self) -> Result<(), Box<dyn Error>> {
        let file_content = AppConfig::default().to_string()?;
        let file_path = self.locations_provider.get_config_file_path();

        let mut file = match File::create(&file_path) {
            Err(err) => {
                return Err(Box::new(err));
            }
            Ok(file) => file,
        };

        Ok(file.write_all(file_content.as_bytes())?)
    }
}
