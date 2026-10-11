use std::path::{Path, PathBuf};

fn contains_assets(directory: &Path) -> bool {
    directory.join("icons").is_dir() && directory.join("audio").is_dir()
}

pub fn resolve_resource_root(executable: &Path) -> Result<PathBuf, String> {
    let parent = executable
        .parent()
        .ok_or_else(|| format!("Executable path has no parent: {}", executable.display()))?;

    if contains_assets(parent) {
        return Ok(parent.to_path_buf());
    }

    parent
        .ancestors()
        .find(|directory| contains_assets(directory) && directory.join("Cargo.toml").is_file())
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            format!(
                "Could not find icons/ and audio/ beside {} or in the project tree",
                executable.display()
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!("med-tracker-resources-{stamp}"));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn add_assets(root: &Path) {
        fs::create_dir_all(root.join("icons")).unwrap();
        fs::create_dir_all(root.join("audio")).unwrap();
    }

    #[test]
    fn resolves_installed_bundle_next_to_executable() {
        let root = TempDir::new();
        add_assets(root.path());

        assert_eq!(
            resolve_resource_root(&root.path().join("Med-Tracker")).unwrap(),
            root.path()
        );
    }

    #[test]
    fn resolves_development_assets_from_a_nested_target_directory() {
        let root = TempDir::new();
        add_assets(root.path());
        fs::write(root.path().join("Cargo.toml"), "[package]").unwrap();
        let executable = root.path().join("target/release/Med-Tracker");

        assert_eq!(resolve_resource_root(&executable).unwrap(), root.path());
    }

    #[test]
    fn rejects_executable_without_a_complete_asset_root() {
        let root = TempDir::new();
        fs::create_dir_all(root.path().join("icons")).unwrap();

        assert!(resolve_resource_root(&root.path().join("Med-Tracker")).is_err());
    }
}
