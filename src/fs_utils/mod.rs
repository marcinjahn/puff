use std::{path::Path, error::Error};

pub fn is_empty_dir(path: &Path) -> Result<bool, Box<dyn Error>> {
    Ok(path.read_dir()?.next().is_none())
}