use crate::cli::{BuildMode, RunArgs};
use crate::godot;
use crate::paths::ProjectPaths;
use crate::process::{self, Program};
use anyhow::Result;

pub fn rust_build_args(build: BuildMode) -> Vec<Vec<String>> {
    match build {
        BuildMode::Debug => vec![vec!["build".into(), "--locked".into()]],
        BuildMode::Release => vec![vec!["build".into(), "--release".into(), "--locked".into()]],
        BuildMode::Both => vec![
            vec!["build".into(), "--release".into(), "--locked".into()],
            vec!["build".into(), "--locked".into()],
        ],
        BuildMode::None => Vec::new(),
    }
}

pub fn godot_launch_args(args: &RunArgs) -> Vec<String> {
    let mut launch = Vec::new();
    if args.headless {
        launch.push("--headless".into());
    }
    launch.push("--path".into());
    launch.push("__GODOT_DIR__".into());
    if args.editor {
        launch.push("--editor".into());
    }
    launch.extend(args.godot_args.clone());
    launch
}

pub fn execute(paths: &ProjectPaths, args: RunArgs) -> Result<()> {
    for cargo_args in rust_build_args(args.build) {
        process::run(&Program::new("cargo"), &paths.rust_dir, &cargo_args)?;
    }

    let godot_exe = godot::resolve_godot_executable(&args.godot_exe)?;
    godot::normalize_extension_list(&paths.godot_dir)?;
    if godot::import_needed(&paths.godot_dir) {
        process::run(
            &Program::new(godot_exe.clone()),
            &paths.repo_root,
            &[
                "--path".into(),
                paths.godot_dir.to_string_lossy().into_owned(),
                "--import".into(),
                "--quit".into(),
            ],
        )?;
        godot::normalize_extension_list(&paths.godot_dir)?;
    }

    let mut launch_args = godot_launch_args(&args);
    for value in &mut launch_args {
        if value == "__GODOT_DIR__" {
            *value = paths.godot_dir.to_string_lossy().into_owned();
        }
    }
    process::run(&Program::new(godot_exe), &paths.repo_root, &launch_args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::BuildMode;

    #[test]
    fn release_build_uses_release_flag_and_lockfile() {
        assert_eq!(
            rust_build_args(BuildMode::Release),
            vec![vec!["build", "--release", "--locked"]]
        );
    }

    #[test]
    fn both_builds_release_before_debug() {
        assert_eq!(
            rust_build_args(BuildMode::Both),
            vec![
                vec!["build", "--release", "--locked"],
                vec!["build", "--locked"]
            ]
        );
    }
}
