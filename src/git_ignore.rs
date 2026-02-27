use anyhow::Result;
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::Write;
use std::{fs::File, path::Path};

/// Adds files to existing/new .gitignore
pub struct GitIgnoreHandler {}

impl GitIgnoreHandler {
    pub fn new() -> Self {
        GitIgnoreHandler {}
    }

    /// An existing .gitignore file in the user_dir will be updated
    /// with the provided file_name. If .gitignore does not exist, it
    /// will be created
    pub fn add_to_git_ignore(&self, user_dir: &Path, file_name: &str) -> Result<GitIgnoreResult> {
        if !user_dir.join(".gitignore").exists() {
            self.create_git_ignore_file(user_dir, file_name)?;

            Ok(GitIgnoreResult::FileCreated)
        } else {
            self.append_to_existing_git_ignore(user_dir, file_name)?;

            Ok(GitIgnoreResult::FileUpdated)
        }
    }

    fn create_git_ignore_file(&self, dir: &Path, file_to_ignore: &str) -> Result<()> {
        let mut file = File::create(dir.join(".gitignore"))?;
        writeln!(file, "{file_to_ignore}")?;

        Ok(())
    }

    fn append_to_existing_git_ignore(&self, dir: &Path, file_to_ignore: &str) -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(dir.join(".gitignore"))?;

        writeln!(file)?; // to make sure we're not appending to some non-empty line
        writeln!(file, "{file_to_ignore}")?;

        Ok(())
    }
}

#[derive(PartialEq, Debug)]
pub enum GitIgnoreResult {
    FileCreated,
    FileUpdated,
}

impl Display for GitIgnoreResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitIgnoreResult::FileCreated => write!(f, "created"),
            GitIgnoreResult::FileUpdated => write!(f, "updated"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::git_ignore::GitIgnoreResult;
    use std::fs::{self, File};
    use std::io::Write;

    use super::GitIgnoreHandler;

    #[test]
    fn add_to_git_ignore_when_git_ignore_does_not_exist() {
        let dir = tempfile::tempdir().unwrap();
        let file_name = "testfile";

        let sut = GitIgnoreHandler::new();
        let result = sut.add_to_git_ignore(dir.path(), file_name).unwrap();

        assert_eq!(GitIgnoreResult::FileCreated, result);

        let git_ignore_path = dir.path().join(".gitignore");
        assert!(git_ignore_path.exists());

        let contents = fs::read_to_string(git_ignore_path).unwrap();
        assert_eq!(format!("{file_name}\n"), contents);
    }

    #[test]
    fn add_to_git_ignore_when_git_ignore_does_exist() {
        let dir = tempfile::tempdir().unwrap();
        let mut file = File::create(dir.path().join(".gitignore")).unwrap();
        write!(file, "some-existing-content").unwrap();

        let file_name = "testfile";

        let sut = GitIgnoreHandler::new();
        let result = sut.add_to_git_ignore(dir.path(), file_name).unwrap();

        assert_eq!(GitIgnoreResult::FileUpdated, result);

        let git_ignore_path = dir.path().join(".gitignore");
        assert!(git_ignore_path.exists());

        let contents = fs::read_to_string(git_ignore_path).unwrap();
        assert_eq!(format!("some-existing-content\n{file_name}\n"), contents);
    }
}
