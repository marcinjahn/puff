use anyhow::{anyhow, Result};
use std::fs;
use std::process::Command;

use crate::config::locations::LocationsProvider;

pub struct CdCommand<'a> {
    locations_provider: &'a LocationsProvider,
}

impl<'a> CdCommand<'a> {
    pub fn new(locations_provider: &'a LocationsProvider) -> Self {
        CdCommand {
            locations_provider,
        }
    }

    pub fn cd(&self, print: bool) -> Result<()> {
        let path = self.locations_provider.get_projects_data_path();

        if !path.exists() {
            fs::create_dir_all(&path)?;
        }

        if print {
            println!("{}", path.display());
            return Ok(());
        }

        let shell = get_shell()?;
        let config_path = self.locations_provider.get_base_config_path()?;
        let data_path = self.locations_provider.get_base_data_path()?;
        let status = Command::new(&shell)
            .current_dir(&path)
            .env("PUFF_SUBSHELL", "1")
            .env("PUFF_CONFIG_PATH", &config_path)
            .env("PUFF_DATA_PATH", &data_path)
            .status()
            .map_err(|e| anyhow!("Failed to spawn shell '{}': {}", shell, e))?;

        std::process::exit(status.code().unwrap_or(1));
    }
}

#[cfg(unix)]
fn get_shell() -> Result<String> {
    std::env::var("SHELL").or_else(|_| Ok("/bin/sh".to_string()))
}

#[cfg(windows)]
fn get_shell() -> Result<String> {
    std::env::var("COMSPEC").or_else(|_| Ok("cmd.exe".to_string()))
}
