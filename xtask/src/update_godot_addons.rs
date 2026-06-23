use crate::cli::{GodotAddonSelection, UpdateGodotAddonsArgs};
use crate::paths::ProjectPaths;
use crate::process;
use anyhow::{Context, Result};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug)]
struct AddonSpec {
    label: &'static str,
    repo_url: &'static str,
    source_dir: &'static str,
    target_dir: &'static str,
}

const LDTK: AddonSpec = AddonSpec {
    label: "ldtk-importer",
    repo_url: "https://github.com/heygleeson/godot-ldtk-importer.git",
    source_dir: "addons/ldtk-importer",
    target_dir: "addons/ldtk-importer",
};

const ASEPRITE: AddonSpec = AddonSpec {
    label: "AsepriteWizard",
    repo_url: "https://github.com/viniciusgerevini/godot-aseprite-wizard.git",
    source_dir: "addons/AsepriteWizard",
    target_dir: "addons/AsepriteWizard",
};

pub fn execute(paths: &ProjectPaths, args: UpdateGodotAddonsArgs) -> Result<()> {
    let addons = selected_addons(args.addon);
    if args.ref_name.is_some() && addons.len() != 1 {
        anyhow::bail!("--ref can only be used with --addon ldtk or --addon aseprite");
    }

    for addon in addons {
        update_addon(paths, addon, args.ref_name.as_deref(), args.dry_run)?;
    }

    Ok(())
}

fn update_addon(
    paths: &ProjectPaths,
    addon: AddonSpec,
    requested_ref: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    let current_version = read_plugin_version(&paths.godot_dir.join(addon.target_dir))
        .unwrap_or_else(|_| "unknown".to_owned());
    let ref_name = match requested_ref {
        Some(value) => value.to_owned(),
        None => fetch_latest_release_tag(addon.repo_url)?,
    };

    if dry_run {
        println!(
            "{}: current {}, target {}",
            addon.label, current_version, ref_name
        );
        return Ok(());
    }

    if requested_ref.is_none() && release_tag_matches_version(&ref_name, &current_version) {
        println!("{}: already at {}", addon.label, current_version);
        return Ok(());
    }

    let temp = tempfile::tempdir().context("failed to create temp download directory")?;
    let repo_dir = download_repo_archive(addon.repo_url, &ref_name, temp.path())?;
    let source = repo_dir.join(addon.source_dir);
    if !source.is_dir() {
        anyhow::bail!(
            "{} not found in {} at {}",
            addon.source_dir,
            addon.repo_url,
            ref_name
        );
    }

    let source_version = read_plugin_version(&source).unwrap_or_else(|_| format!("ref {ref_name}"));
    if requested_ref.is_none() && current_version == source_version {
        println!("{}: already at {}", addon.label, current_version);
        return Ok(());
    }

    let target = paths.godot_dir.join(addon.target_dir);
    replace_dir(&source, &target)?;
    println!(
        "{}: updated {} -> {}",
        addon.label, current_version, source_version
    );
    Ok(())
}

fn release_tag_matches_version(tag: &str, version: &str) -> bool {
    let tag = tag.trim_start_matches('v');
    tag == version
        || tag
            .strip_prefix(version)
            .is_some_and(|rest| matches!(rest.as_bytes().first(), Some(b'-' | b'+' | b'_')))
}

fn selected_addons(selection: GodotAddonSelection) -> Vec<AddonSpec> {
    match selection {
        GodotAddonSelection::All => vec![LDTK, ASEPRITE],
        GodotAddonSelection::Ldtk => vec![LDTK],
        GodotAddonSelection::Aseprite => vec![ASEPRITE],
    }
}

fn fetch_latest_release_tag(repo_url: &str) -> Result<String> {
    let api_url = github_api_latest_release_url(repo_url)
        .with_context(|| format!("unsupported GitHub URL: {repo_url}"))?;
    let response: serde_json::Value = ureq::get(&api_url)
        .set("User-Agent", "p1proto-xtask")
        .call()
        .with_context(|| format!("GitHub API request failed: {api_url}"))?
        .into_json()
        .context("failed to parse GitHub API response")?;

    response["tag_name"]
        .as_str()
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .context("GitHub latest release response has no tag_name")
}

fn download_repo_archive(repo_url: &str, ref_name: &str, temp_dir: &Path) -> Result<PathBuf> {
    let api_url = github_api_zipball_url(repo_url, ref_name)
        .with_context(|| format!("unsupported GitHub URL: {repo_url}"))?;
    let archive_path = temp_dir.join("repo.zip");
    let mut reader = ureq::get(&api_url)
        .set("User-Agent", "p1proto-xtask")
        .call()
        .with_context(|| format!("GitHub archive request failed: {api_url}"))?
        .into_reader();
    let mut archive = fs::File::create(&archive_path)
        .with_context(|| format!("failed to create {}", archive_path.display()))?;
    io::copy(&mut reader, &mut archive).context("failed to save GitHub archive")?;

    let extract_dir = temp_dir.join("extract");
    fs::create_dir(&extract_dir)
        .with_context(|| format!("failed to create {}", extract_dir.display()))?;
    process::run(
        &crate::process::Program::new("tar"),
        temp_dir,
        &[
            "-xf".into(),
            "repo.zip".into(),
            "-C".into(),
            "extract".into(),
        ],
    )?;

    single_child_dir(&extract_dir)
}

fn github_api_latest_release_url(repo_url: &str) -> Option<String> {
    let (owner, repo) = github_repo_path(repo_url)?;
    Some(format!(
        "https://api.github.com/repos/{owner}/{repo}/releases/latest"
    ))
}

fn github_api_zipball_url(repo_url: &str, ref_name: &str) -> Option<String> {
    let (owner, repo) = github_repo_path(repo_url)?;
    Some(format!(
        "https://api.github.com/repos/{owner}/{repo}/zipball/{ref_name}"
    ))
}

fn github_repo_path(repo_url: &str) -> Option<(&str, &str)> {
    let trimmed = repo_url.trim_end_matches('/');
    let path = if let Some(rest) = trimmed.strip_prefix("https://github.com/") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("git@github.com:") {
        rest
    } else {
        return None;
    };
    let path = path.strip_suffix(".git").unwrap_or(path);
    path.split_once('/')
}

fn replace_dir(source: &Path, target: &Path) -> Result<()> {
    let parent = target
        .parent()
        .with_context(|| format!("{} has no parent", target.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;

    let file_name = target
        .file_name()
        .context("target directory has no file name")?
        .to_string_lossy();
    let temp_target = parent.join(format!(".{file_name}.upgrade-tmp"));
    if temp_target.exists() {
        fs::remove_dir_all(&temp_target)
            .with_context(|| format!("failed to remove {}", temp_target.display()))?;
    }

    copy_dir_recursive(source, &temp_target)?;
    if target.exists() {
        fs::remove_dir_all(target)
            .with_context(|| format!("failed to remove {}", target.display()))?;
    }
    fs::rename(&temp_target, target).with_context(|| {
        format!(
            "failed to move {} to {}",
            temp_target.display(),
            target.display()
        )
    })?;
    Ok(())
}

fn copy_dir_recursive(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target).with_context(|| format!("failed to create {}", target.display()))?;
    for entry in
        fs::read_dir(source).with_context(|| format!("failed to read {}", source.display()))?
    {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path).with_context(|| {
                format!(
                    "failed to copy {} to {}",
                    source_path.display(),
                    target_path.display()
                )
            })?;
        }
    }
    Ok(())
}

fn read_plugin_version(addon_dir: &Path) -> Result<String> {
    let cfg = fs::read_to_string(addon_dir.join("plugin.cfg"))
        .with_context(|| format!("failed to read {}", addon_dir.join("plugin.cfg").display()))?;
    cfg.lines()
        .find_map(|line| line.trim().strip_prefix("version="))
        .map(|value| value.trim_matches('"').to_owned())
        .context("plugin.cfg has no version")
}

fn single_child_dir(parent: &Path) -> Result<PathBuf> {
    let dirs = fs::read_dir(parent)
        .with_context(|| format!("failed to read {}", parent.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();

    match dirs.as_slice() {
        [dir] => Ok(dir.clone()),
        [] => anyhow::bail!("{} has no extracted directory", parent.display()),
        _ => anyhow::bail!("{} has multiple extracted directories", parent.display()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn converts_github_urls_to_latest_release_api_urls() {
        assert_eq!(
            github_api_latest_release_url("https://github.com/heygleeson/godot-ldtk-importer.git"),
            Some(
                "https://api.github.com/repos/heygleeson/godot-ldtk-importer/releases/latest"
                    .to_owned()
            )
        );
        assert_eq!(
            github_api_latest_release_url("git@github.com:viniciusgerevini/godot-aseprite-wizard.git"),
            Some(
                "https://api.github.com/repos/viniciusgerevini/godot-aseprite-wizard/releases/latest"
                    .to_owned()
            )
        );
        assert_eq!(
            github_api_zipball_url(
                "https://github.com/viniciusgerevini/godot-aseprite-wizard.git",
                "v9.8.0-4"
            ),
            Some(
                "https://api.github.com/repos/viniciusgerevini/godot-aseprite-wizard/zipball/v9.8.0-4"
                    .to_owned()
            )
        );
    }

    #[test]
    fn replace_dir_copies_nested_files_and_removes_old_files() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let target = temp.path().join("target");
        fs::create_dir_all(source.join("nested")).unwrap();
        fs::create_dir_all(&target).unwrap();
        fs::write(source.join("plugin.cfg"), "version=\"1\"\n").unwrap();
        fs::write(source.join("nested/file.gd"), "new").unwrap();
        fs::write(target.join("old.gd"), "old").unwrap();

        replace_dir(&source, &target).unwrap();

        assert_eq!(
            fs::read_to_string(target.join("nested/file.gd")).unwrap(),
            "new"
        );
        assert!(!target.join("old.gd").exists());
        assert_eq!(read_plugin_version(&target).unwrap(), "1");
    }

    #[test]
    fn selected_single_addon_is_exact() {
        let addons = selected_addons(GodotAddonSelection::Aseprite);
        assert_eq!(addons.len(), 1);
        assert_eq!(
            PathBuf::from(addons[0].target_dir),
            PathBuf::from("addons/AsepriteWizard")
        );
    }

    #[test]
    fn release_tags_match_plugin_versions() {
        assert!(release_tag_matches_version("2.0.1", "2.0.1"));
        assert!(release_tag_matches_version("v9.8.0-4", "9.8.0"));
        assert!(!release_tag_matches_version("2.0.1", "2.0"));
    }
}
