use crate::cli::{ExportArgs, ExportTarget};
use crate::godot;
use crate::paths::ProjectPaths;
use crate::process::{self, Program};
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub fn export_output_path(paths: &ProjectPaths, target: ExportTarget) -> PathBuf {
    match target {
        ExportTarget::Windows => paths.export_dir.join("p1proto.exe"),
        ExportTarget::Macos => paths.export_dir.join("p1proto.zip"),
    }
}

pub fn rust_build_args(skip_debug_build: bool) -> Vec<Vec<String>> {
    let mut args = vec![vec!["build".into(), "--release".into(), "--locked".into()]];
    if !skip_debug_build {
        args.push(vec!["build".into(), "--locked".into()]);
    }
    args
}

pub fn ensure_export_preset(
    godot_dir: &std::path::Path,
    preset_name: &str,
    force: bool,
) -> Result<()> {
    let path = godot_dir.join("export_presets.cfg");
    if path.is_file() {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        if content.contains(&format!("name=\"{preset_name}\"")) {
            return Ok(());
        }
        if !force {
            anyhow::bail!(
                "export preset '{preset_name}' not found in {}; pass --force-create-export-preset",
                path.display()
            );
        }
    }

    let content = format!(
        "[preset.0]\nname=\"{preset_name}\"\nplatform=\"Windows Desktop\"\nrunnable=true\nadvanced_options=false\n\n"
    );
    fs::write(&path, content).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub fn execute(paths: &ProjectPaths, args: ExportArgs) -> Result<()> {
    for cargo_args in rust_build_args(args.skip_debug_build) {
        process::run(&Program::new("cargo"), &paths.rust_dir, &cargo_args)?;
    }

    fs::create_dir_all(&paths.export_dir)
        .with_context(|| format!("failed to create {}", paths.export_dir.display()))?;
    ensure_export_preset(
        &paths.godot_dir,
        &args.preset_name,
        args.force_create_export_preset,
    )?;
    godot::normalize_extension_list(&paths.godot_dir)?;

    let godot_exe = godot::resolve_godot_executable(&args.godot_exe)?;
    let output = export_output_path(paths, args.target);
    process::run(
        &Program::new(godot_exe),
        &paths.repo_root,
        &[
            "--headless".into(),
            "--path".into(),
            paths.godot_dir.to_string_lossy().into_owned(),
            "--export-release".into(),
            args.preset_name,
            output.to_string_lossy().into_owned(),
        ],
    )?;

    if output.exists() {
        Ok(())
    } else {
        anyhow::bail!("export failed: output not found at {}", output.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::ExportTarget;
    use crate::paths::ProjectPaths;
    use std::fs;
    use tempfile::TempDir;

    fn paths(root: &std::path::Path) -> ProjectPaths {
        ProjectPaths {
            repo_root: root.to_path_buf(),
            rust_dir: root.join("rust"),
            godot_dir: root.join("godot"),
            export_dir: root.join("export"),
        }
    }

    #[test]
    fn windows_export_writes_exe_and_macos_writes_zip() {
        let temp = TempDir::new().unwrap();
        let paths = paths(temp.path());
        assert_eq!(
            export_output_path(&paths, ExportTarget::Windows),
            temp.path().join("export/p1proto.exe")
        );
        assert_eq!(
            export_output_path(&paths, ExportTarget::Macos),
            temp.path().join("export/p1proto.zip")
        );
    }

    #[test]
    fn release_build_is_always_first() {
        assert_eq!(
            rust_build_args(false),
            vec![
                vec!["build", "--release", "--locked"],
                vec!["build", "--locked"]
            ]
        );
        assert_eq!(
            rust_build_args(true),
            vec![vec!["build", "--release", "--locked"]]
        );
    }

    #[test]
    fn force_creates_export_presets_file() {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("godot")).unwrap();
        ensure_export_preset(&temp.path().join("godot"), "Windows Desktop", true).unwrap();
        let content = fs::read_to_string(temp.path().join("godot/export_presets.cfg")).unwrap();
        assert!(content.contains("name=\"Windows Desktop\""));
    }
}
