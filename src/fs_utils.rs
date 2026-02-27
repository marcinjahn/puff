use anyhow::{Result, anyhow};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Creates a backup of a file in the same directory. It adds ".bak"
/// suffix to the backup file. If it creates a conflict "1"s will be
/// added to the name until there is no conflict.
pub fn backup_file(file_path: &Path) -> Result<Option<String>> {
    let backup_path = get_backup_path(file_path)?;
    fs::copy(file_path, &backup_path)?;

    Ok(Some(backup_path.to_str().unwrap().to_string()))
}

pub fn get_backup_path(file_path: &Path) -> Result<PathBuf> {
    let mut path_string = file_path
        .to_str()
        .ok_or_else(|| anyhow!("A path of a file to backup could not be converted to a string"))?
        .to_owned();

    let mut bak = path_string + ".bak";
    let mut backup_path = Path::new(&bak);
    while backup_path.exists() {
        path_string = backup_path.to_str().unwrap().to_owned();
        bak = path_string + "1";
        backup_path = Path::new(&bak);
    }

    Ok(backup_path.to_path_buf())
}

pub fn is_symlink(user_file: &Path) -> Result<bool> {
    let metadata = fs::symlink_metadata(user_file)?;

    Ok(metadata.is_symlink())
}

pub fn symlink_file(original: impl AsRef<Path>, link: impl AsRef<Path>) -> Result<()> {
    #[cfg(unix)]
    std::os::unix::fs::symlink(original, link)?;
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(original, link)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::File;

    use super::backup_file;
    use std::io::Write;

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
