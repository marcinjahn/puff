use std::{fs, path::PathBuf};

pub(crate) fn get_config_without_whitespace(path: PathBuf) -> String {
    fs::read_to_string(path)
        .unwrap()
        .replace("\n", "")
        .replace("\r", "")
        .replace(" ", "")
}
