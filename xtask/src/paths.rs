use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectPaths {
    pub repo_root: PathBuf,
    pub rust_dir: PathBuf,
    pub godot_dir: PathBuf,
    pub export_dir: PathBuf,
}

impl ProjectPaths {
    pub fn discover() -> Result<Self> {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir
            .parent()
            .context("xtask manifest directory has no parent")?
            .to_path_buf();
        Self::from_repo_root(repo_root)
    }

    pub fn from_repo_root(repo_root: PathBuf) -> Result<Self> {
        let rust_dir = repo_root.join("rust");
        let godot_dir = repo_root.join("godot");
        let export_dir = repo_root.join("export");
        require_file(&rust_dir.join("Cargo.toml"))?;
        require_file(&godot_dir.join("project.godot"))?;
        Ok(Self {
            repo_root,
            rust_dir,
            godot_dir,
            export_dir,
        })
    }
}

fn require_file(path: &Path) -> Result<()> {
    if path.is_file() {
        Ok(())
    } else {
        anyhow::bail!("required file not found: {}", path.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn discovers_standard_project_layout() {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("rust")).unwrap();
        fs::create_dir(temp.path().join("godot")).unwrap();
        fs::write(temp.path().join("rust/Cargo.toml"), "[package]\nname='rust'\n").unwrap();
        fs::write(temp.path().join("godot/project.godot"), "; godot\n").unwrap();

        let paths = ProjectPaths::from_repo_root(temp.path().to_path_buf()).unwrap();
        assert_eq!(paths.rust_dir, temp.path().join("rust"));
        assert_eq!(paths.godot_dir, temp.path().join("godot"));
        assert_eq!(paths.export_dir, temp.path().join("export"));
    }
}
