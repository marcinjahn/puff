use std::{error::Error, path::{PathBuf}, env};

use crate::secret_files;

pub fn add_file(mut file_path: PathBuf) -> Result<(), Box<dyn Error>> {
    if !file_path.is_absolute() {
        let cwd = env::current_dir()?;
        file_path = cwd.join(file_path);
    }

    let project_name = secret_files::add_file(&file_path)?;
    let file_name = file_path.file_name().unwrap();

    println!("The file {:?} has been added to the project named '{}'", file_name, project_name);

    Ok(())
}