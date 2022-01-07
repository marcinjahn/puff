use self::config_file::AppConfig;
use directories::ProjectDirs;
use std::{error::Error, fmt, fs::{self, File}, path::Path, io::Write};

pub mod config_file;

const APP_NAME: &str = "conman";
const CONFIG_FILE_NAME: &str = "config.json";

pub struct AppInitializer;

impl AppInitializer {
    pub fn init() -> Result<(), Box<dyn Error>> {
        match ProjectDirs::from("com", "marcinjahn", APP_NAME) {
            Some(dirs) => {
                let path = dirs.config_dir();

                if !path.exists() {
                    AppInitializer::create_app_files(path)?
                }

                Ok(())
            },
            None => Err(Box::new(InitError("Initialization of conman failed. Could not find the path of configuration files of the host.".into())))
        }
    }

    fn create_app_files(base_path: &Path) -> Result<(), Box<dyn Error>> {
        let configs_path = base_path.join(Path::new("configs"));
        match configs_path.to_str() {
            Some(configs_path) => {
                fs::create_dir_all(configs_path)?;
                AppInitializer::create_config_file(base_path)?;
            }
            None => {
                return Err(Box::new(InitError(
                    "Initialization of conman failed. Could not create configuration directory."
                        .into(),
                )))
            }
        }

        Ok(())
    }

    fn create_config_file(base_path: &Path) -> Result<(), Box<dyn Error>> {
        let file_content = AppConfig::default().to_string()?;
        let file_path = base_path.join(Path::new(CONFIG_FILE_NAME));

        let mut file = match File::create(&file_path) {
            Err(err) => {return Err(Box::new(err));},
            Ok(file) => file,
        };
    
        Ok(file.write_all(file_content.as_bytes())?)
    }
}

#[derive(Debug)]
struct InitError(String);

impl Error for InitError {}

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "conman initialization failed")
    }
}
