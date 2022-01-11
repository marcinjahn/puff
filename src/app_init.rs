use crate::config::app_config::AppConfig;
use crate::config::locations;
use std::{
    error::Error,
    fs::{self, File},
    io::Write,
};

pub fn init() -> Result<(), Box<dyn Error>> {
    let config_path = locations::get_base_config_path()?;
    if !config_path.exists() {
        create_app_files()?
    }

    Ok(())
}

fn create_app_files() -> Result<(), Box<dyn Error>> {
    let configs_path = locations::get_configs_config_path()?;
    fs::create_dir_all(configs_path)?;
    create_config_file()?;

    Ok(())
}

fn create_config_file() -> Result<(), Box<dyn Error>> {
    let file_content = AppConfig::default().to_string()?;
    let file_path = locations::get_config_file_path()?;

    let mut file = match File::create(&file_path) {
        Err(err) => {
            return Err(Box::new(err));
        }
        Ok(file) => file,
    };

    Ok(file.write_all(file_content.as_bytes())?)
}
