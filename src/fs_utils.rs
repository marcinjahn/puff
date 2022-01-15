use std::{path::Path, error::Error, fs};

/// checks if a directory is empty
pub fn is_empty_dir(path: &Path) -> Result<bool, Box<dyn Error>> {
    Ok(path.read_dir()?.next().is_none())
}

/// Creates a backup of a file in the same directory. It adds ".bak"
/// suffix to the backup file. If it creates a conflict "1"s will be
/// added to the name until there is no conflict.
pub fn backup_file(file_path: &Path) -> Result<Option<String>, Box<dyn Error>> {
    let mut path_string = file_path
        .to_str()
        .ok_or("A path of a file to backup could not be converted to a string")?
        .to_owned();
    let mut bak = path_string + ".bak";
    let mut backup_path = Path::new(&bak);

    while backup_path.exists() {
        path_string = backup_path.to_str().unwrap().to_owned();
        bak = path_string + "1";
        backup_path = Path::new(&bak);
    }
    
    fs::copy(file_path, backup_path)?;

    Ok(Some(backup_path.to_str().unwrap().to_string()))
}



#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::fs;

    use super::{is_empty_dir, backup_file};
    use std::io::Write;

    #[test]
    fn is_empty_dir_when_dir_is_empty_then_returns_true() {
        let dir = tempfile::tempdir().unwrap();
        
        let result = is_empty_dir(dir.path()).unwrap();

        assert!(result);
    }

    #[test]
    fn is_empty_dir_when_dir_is_not_empty_then_returns_false() {
        let dir = tempfile::tempdir().unwrap();
        let _file = File::create(dir.path().join("some-file")).unwrap();
        
        let result = is_empty_dir(dir.path()).unwrap();

        assert!(!result);
    }

    #[test]
    fn is_empty_dir_when_dir_doesnt_exist_then_err_is_returned() {
        let dir = tempfile::tempdir().unwrap();
        let non_existing_dir = dir.path().join("i-dont-exist");
        
        let result = is_empty_dir(&non_existing_dir);

        result.unwrap_err();
    }

    #[test]
    fn backup_file_when_source_file_does_not_exist_then_err_is_returned() {
        let dir = tempfile::tempdir().unwrap();
        let non_existing_file = dir.path().join("i-dont-exist");

        backup_file(&non_existing_file).unwrap_err();
    }
    
    #[test]
    fn backup_file_when_source_file_exists_then_backup_gets_created() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("some-file");
        let mut file = File::create(&file_path).unwrap();
        write!(file, "some content").unwrap();

        backup_file(&file_path).unwrap();

        let backup_path = file_path.with_extension("bak");
        let backup_content = fs::read_to_string(backup_path).unwrap();

        assert_eq!("some content", backup_content);
    }

    #[test]
    fn backup_file_when_source_file_exists_and_backup_existis_then_new_backup_gets_created() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("some-file");
        let mut file = File::create(&file_path).unwrap();
        write!(file, "some content").unwrap();

        let existing_backup_path = file_path.with_extension("bak");
        let mut existing_backup_file = File::create(&existing_backup_path).unwrap();
        write!(existing_backup_file, "some content").unwrap();

        backup_file(&file_path).unwrap();

        let backup_path = file_path.with_extension("bak1");
        let backup_content = fs::read_to_string(backup_path).unwrap();

        assert_eq!("some content", backup_content);
    }
}