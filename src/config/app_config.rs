use std::{path::PathBuf, error::Error};
use serde::{Deserialize, Serialize};

#[derive(Default)]
#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    projects: Vec<Project>
}

impl AppConfig {
    pub fn to_string(&self) -> Result<String, Box<dyn Error>> {
        Ok(serde_json::to_string(self)?)
    }
}


#[derive(Serialize, Deserialize)]
struct Project {
    name: String,
    id: String,
    path: PathBuf
}
