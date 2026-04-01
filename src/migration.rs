use crate::config::{app_config::AppConfigManager, locations::LocationsProvider};
use crate::fs_utils::copy_dir_recursive;
use anyhow::{Result, bail};
use std::fs;
use std::path::Path;

/// Migrates the legacy `{config_path}/configs/` directory to `{data_path}/projects/`.
/// Returns `Ok(true)` if migration was performed, `Ok(false)` if not needed.
pub fn migrate_projects_path_if_needed(locations_provider: &LocationsProvider) -> Result<bool> {
    let legacy_path = locations_provider.get_legacy_configs_path();
    if !legacy_path.exists() {
        return Ok(false);
    }

    let new_path = locations_provider.get_projects_data_path();
    if new_path.exists() {
        bail!(
            "Both legacy '{}' and new '{}' directories exist. \
             Please remove one of them manually to resolve the ambiguity.",
            legacy_path.display(),
            new_path.display()
        );
    }

    // a bit unrealistic to check this, but why not be exxxxtra safe?
    if let Some(parent) = new_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Try atomic rename first (works on same filesystem)
    if fs::rename(&legacy_path, &new_path).is_err() {
        copy_dir_recursive(&legacy_path, &new_path)?;
        fs::remove_dir_all(&legacy_path)?;
    }

    repoint_symlinks(locations_provider, &legacy_path, &new_path);

    println!(
        "Managed projects have been migrated to '{}'. You're all set!",
        new_path.display()
    );

    Ok(true)
}

/// Best-effort: walk each associated project's user directory and repoint
/// symlinks that targeted the old configs/ path to the new projects/ path.
fn repoint_symlinks(locations_provider: &LocationsProvider, old_base: &Path, new_base: &Path) {
    let config_file = locations_provider.get_config_file_path();
    let Ok(config_manager) = AppConfigManager::new(config_file) else {
        return;
    };
    let Ok(config) = config_manager.get_config() else {
        return;
    };

    for project in &config.projects {
        repoint_project_symlinks(&project.path, &project.name, old_base, new_base);
    }
}

fn repoint_project_symlinks(user_dir: &Path, project_name: &str, old_base: &Path, new_base: &Path) {
    let old_managed = old_base.join(project_name);
    let new_managed = new_base.join(project_name);

    let Ok(entries) = walk_files(user_dir) else {
        return;
    };

    for file_path in entries {
        let Ok(target) = fs::read_link(&file_path) else {
            continue;
        };
        if let Ok(relative) = target.strip_prefix(&old_managed) {
            let new_target = new_managed.join(relative);

            let _ = fs::remove_file(&file_path);
            #[cfg(unix)]
            let _ = std::os::unix::fs::symlink(&new_target, &file_path);
            #[cfg(windows)]
            let _ = std::os::windows::fs::symlink_file(&new_target, &file_path);
        }
    }
}

fn walk_files(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut files = vec![];
    let mut dirs = vec![dir.to_owned()];

    while let Some(current) = dirs.pop() {
        let Ok(entries) = fs::read_dir(&current) else {
            continue;
        };

        for entry in entries {
            let Ok(entry) = entry else { continue };
            let path = entry.path();
            if path.is_dir() && !path.is_symlink() {
                dirs.push(path);
            } else {
                files.push(path);
            }
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::locations::LocationsProvider;
    use std::fs;
    use std::io::Write;

    fn setup_locations(config_dir: &Path, data_dir: &Path) -> LocationsProvider {
        LocationsProvider::new(config_dir.to_path_buf(), data_dir.to_path_buf())
    }

    fn write_config(config_dir: &Path, json: &str) {
        fs::create_dir_all(config_dir).unwrap();
        let config_file = config_dir.join("config.json");
        let mut f = fs::File::create(config_file).unwrap();
        f.write_all(json.as_bytes()).unwrap();
    }

    #[test]
    fn no_migration_when_no_legacy_dir() {
        let config_dir = tempfile::tempdir().unwrap();
        let data_dir = tempfile::tempdir().unwrap();
        write_config(config_dir.path(), r#"{"projects":[]}"#);

        let lp = setup_locations(config_dir.path(), data_dir.path());
        let result = migrate_projects_path_if_needed(&lp).unwrap();

        assert!(!result);
    }

    #[test]
    fn successful_migration() {
        let config_dir = tempfile::tempdir().unwrap();
        let data_dir = tempfile::tempdir().unwrap();
        write_config(config_dir.path(), r#"{"projects":[]}"#);

        let legacy = config_dir.path().join("configs");
        fs::create_dir_all(legacy.join("myproject")).unwrap();
        fs::write(legacy.join("myproject/.env"), "SECRET=1").unwrap();

        let lp = setup_locations(config_dir.path(), data_dir.path());
        let result = migrate_projects_path_if_needed(&lp).unwrap();

        assert!(result);
        assert!(!legacy.exists());
        let new_path = data_dir.path().join("projects/myproject/.env");
        assert!(new_path.exists());
        assert_eq!(fs::read_to_string(new_path).unwrap(), "SECRET=1");
    }

    #[test]
    fn fails_when_both_dirs_exist() {
        let config_dir = tempfile::tempdir().unwrap();
        let data_dir = tempfile::tempdir().unwrap();
        write_config(config_dir.path(), r#"{"projects":[]}"#);

        fs::create_dir_all(config_dir.path().join("configs/proj")).unwrap();
        fs::create_dir_all(data_dir.path().join("projects/proj")).unwrap();

        let lp = setup_locations(config_dir.path(), data_dir.path());
        let result = migrate_projects_path_if_needed(&lp);

        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("Both legacy"));
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_repointed_after_migration() {
        let config_dir = tempfile::tempdir().unwrap();
        let data_dir = tempfile::tempdir().unwrap();
        let user_dir = tempfile::tempdir().unwrap();

        write_config(
            config_dir.path(),
            &format!(
                r#"{{"projects":[{{"name":"proj","id":"1","path":"{}"}}]}}"#,
                user_dir.path().display()
            ),
        );

        let legacy = config_dir.path().join("configs");
        fs::create_dir_all(legacy.join("proj")).unwrap();
        fs::write(legacy.join("proj/.env"), "SECRET=1").unwrap();

        // Create symlink in user dir pointing to old location
        let symlink_path = user_dir.path().join(".env");
        std::os::unix::fs::symlink(legacy.join("proj/.env"), &symlink_path).unwrap();

        let lp = setup_locations(config_dir.path(), data_dir.path());
        migrate_projects_path_if_needed(&lp).unwrap();

        // Symlink should now point to new location
        let target = fs::read_link(&symlink_path).unwrap();
        assert!(target.starts_with(data_dir.path().join("projects")));
        assert_eq!(fs::read_to_string(&symlink_path).unwrap(), "SECRET=1");
    }
}
