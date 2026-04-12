use anyhow::Result;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

const MANAGED_DIRS_FILE: &str = ".puff_managed_dirs";

pub fn managed_dirs_filename() -> &'static str {
    MANAGED_DIRS_FILE
}

fn managed_dirs_path(managed_dir: &Path) -> PathBuf {
    managed_dir.join(MANAGED_DIRS_FILE)
}

pub fn read_managed_dirs(managed_dir: &Path) -> Result<Vec<PathBuf>> {
    let path = managed_dirs_path(managed_dir);
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = fs::read_to_string(&path)?;
    Ok(content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(PathBuf::from)
        .collect())
}

pub fn add_managed_dir(managed_dir: &Path, relative_dir: &Path) -> Result<()> {
    let mut dirs = read_managed_dirs(managed_dir)?;
    if !dirs.contains(&relative_dir.to_path_buf()) {
        dirs.push(relative_dir.to_path_buf());
        write_managed_dirs(managed_dir, &dirs)?;
    }
    Ok(())
}

pub fn remove_managed_dir(managed_dir: &Path, relative_dir: &Path) -> Result<()> {
    let mut dirs = read_managed_dirs(managed_dir)?;
    dirs.retain(|d| d != relative_dir);
    write_managed_dirs(managed_dir, &dirs)?;
    Ok(())
}

/// Returns the managed ancestor directory if the given relative path resides inside one, or None.
pub fn is_inside_managed_dir(managed_dir: &Path, relative_path: &Path) -> Result<Option<PathBuf>> {
    let dirs = read_managed_dirs_set(managed_dir)?;
    Ok(find_managed_ancestor(&dirs, relative_path))
}

pub enum PathClassification {
    IsManaged,
    InsideManaged(PathBuf),
    Unmanaged,
}

/// Reads `.puff_managed_dirs` once and classifies a path as either a managed directory itself,
/// inside a managed directory, or unmanaged.
pub fn classify_path(managed_dir: &Path, relative_path: &Path) -> Result<PathClassification> {
    let dirs = read_managed_dirs_set(managed_dir)?;
    if dirs.contains(&relative_path.to_path_buf()) {
        return Ok(PathClassification::IsManaged);
    }
    if let Some(ancestor) = find_managed_ancestor(&dirs, relative_path) {
        return Ok(PathClassification::InsideManaged(ancestor));
    }
    Ok(PathClassification::Unmanaged)
}

pub fn read_managed_dirs_set(managed_dir: &Path) -> Result<HashSet<PathBuf>> {
    Ok(read_managed_dirs(managed_dir)?.into_iter().collect())
}

fn find_managed_ancestor(dirs: &HashSet<PathBuf>, relative_path: &Path) -> Option<PathBuf> {
    for ancestor in relative_path.ancestors() {
        if ancestor == Path::new("") {
            break;
        }
        if dirs.contains(&ancestor.to_path_buf()) {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}

fn write_managed_dirs(managed_dir: &Path, dirs: &[PathBuf]) -> Result<()> {
    let content = dirs
        .iter()
        .map(|d| d.display().to_string())
        .collect::<Vec<_>>()
        .join("\n");
    let path = managed_dirs_path(managed_dir);
    fs::write(
        &path,
        if content.is_empty() {
            String::new()
        } else {
            content + "\n"
        },
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn read_returns_empty_when_file_missing() {
        let dir = tempfile::tempdir().unwrap();
        let result = read_managed_dirs(dir.path()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn add_and_read_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        add_managed_dir(dir.path(), Path::new("config")).unwrap();
        add_managed_dir(dir.path(), Path::new("secrets/nested")).unwrap();
        let dirs = read_managed_dirs(dir.path()).unwrap();
        assert_eq!(
            dirs,
            vec![PathBuf::from("config"), PathBuf::from("secrets/nested")]
        );
    }

    #[test]
    fn add_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        add_managed_dir(dir.path(), Path::new("config")).unwrap();
        add_managed_dir(dir.path(), Path::new("config")).unwrap();
        let dirs = read_managed_dirs(dir.path()).unwrap();
        assert_eq!(dirs.len(), 1);
    }

    #[test]
    fn remove_works() {
        let dir = tempfile::tempdir().unwrap();
        add_managed_dir(dir.path(), Path::new("config")).unwrap();
        add_managed_dir(dir.path(), Path::new("secrets")).unwrap();
        remove_managed_dir(dir.path(), Path::new("config")).unwrap();
        let dirs = read_managed_dirs(dir.path()).unwrap();
        assert_eq!(dirs, vec![PathBuf::from("secrets")]);
    }

    #[test]
    fn is_inside_managed_dir_finds_ancestor() {
        let dir = tempfile::tempdir().unwrap();
        add_managed_dir(dir.path(), Path::new("config")).unwrap();
        let result = is_inside_managed_dir(dir.path(), Path::new("config/db.env")).unwrap();
        assert_eq!(result, Some(PathBuf::from("config")));
    }

    #[test]
    fn is_inside_managed_dir_returns_none_when_not_inside() {
        let dir = tempfile::tempdir().unwrap();
        add_managed_dir(dir.path(), Path::new("config")).unwrap();
        let result = is_inside_managed_dir(dir.path(), Path::new("other/file.env")).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn classify_path_works() {
        let dir = tempfile::tempdir().unwrap();
        add_managed_dir(dir.path(), Path::new("config")).unwrap();
        assert!(matches!(
            classify_path(dir.path(), Path::new("config")).unwrap(),
            PathClassification::IsManaged
        ));
        assert!(matches!(
            classify_path(dir.path(), Path::new("config/db.env")).unwrap(),
            PathClassification::InsideManaged(_)
        ));
        assert!(matches!(
            classify_path(dir.path(), Path::new("other")).unwrap(),
            PathClassification::Unmanaged
        ));
    }
}
