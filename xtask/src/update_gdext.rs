use crate::cli::UpdateGdextArgs;
use crate::paths::ProjectPaths;
use crate::process;
use anyhow::{Context, Result};
use std::fs;
use toml_edit::{DocumentMut, value};

pub fn github_api_commit_url(repo_url: &str, branch: &str) -> Option<String> {
    let trimmed = repo_url.trim_end_matches('/');
    let path = if let Some(rest) = trimmed.strip_prefix("https://github.com/") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("git@github.com:") {
        rest
    } else {
        return None;
    };
    let path = path.strip_suffix(".git").unwrap_or(path);
    let (owner, repo) = path.split_once('/')?;
    Some(format!(
        "https://api.github.com/repos/{owner}/{repo}/commits/{branch}"
    ))
}

pub fn set_godot_rev(cargo_toml: &str, rev: &str) -> Result<String> {
    let mut doc = cargo_toml
        .parse::<DocumentMut>()
        .context("failed to parse Cargo.toml")?;
    let godot = &mut doc["dependencies"]["godot"];
    if !godot.is_inline_table() {
        anyhow::bail!("dependencies.godot must be an inline table");
    }
    godot["rev"] = value(rev);
    Ok(doc.to_string())
}

pub fn read_godot_rev(cargo_toml: &str) -> Result<String> {
    let doc = cargo_toml
        .parse::<DocumentMut>()
        .context("failed to parse Cargo.toml")?;
    doc["dependencies"]["godot"]["rev"]
        .as_str()
        .map(str::to_owned)
        .context("dependencies.godot.rev is missing")
}

pub fn fetch_latest_rev(repo_url: &str, branch: &str) -> Result<String> {
    if let Some(api_url) = github_api_commit_url(repo_url, branch) {
        let response: serde_json::Value = ureq::get(&api_url)
            .set("User-Agent", "p1proto-xtask")
            .call()
            .with_context(|| format!("GitHub API request failed: {api_url}"))?
            .into_json()
            .context("failed to parse GitHub API response")?;
        if let Some(rev) = response["sha"].as_str().filter(|rev| is_sha(rev)) {
            return Ok(rev.to_owned());
        }
    }

    let stdout = process::capture_stdout(
        "git",
        std::path::Path::new("."),
        &["ls-remote", repo_url, &format!("refs/heads/{branch}")],
    )?;
    let rev = stdout
        .split_whitespace()
        .next()
        .context("git ls-remote returned no revision")?;
    if is_sha(rev) {
        Ok(rev.to_owned())
    } else {
        anyhow::bail!("git ls-remote returned invalid revision: {rev}")
    }
}

pub fn execute(paths: &ProjectPaths, args: UpdateGdextArgs) -> Result<()> {
    let cargo_toml_path = paths.rust_dir.join("Cargo.toml");
    let original = fs::read_to_string(&cargo_toml_path)
        .with_context(|| format!("failed to read {}", cargo_toml_path.display()))?;
    let current = read_godot_rev(&original)?;
    let latest = fetch_latest_rev(&args.repo_url, &args.branch)?;

    if current == latest {
        println!("godot-rust is already at {latest}");
        return Ok(());
    }

    let updated = set_godot_rev(&original, &latest)?;
    if args.dry_run {
        println!("would update godot-rust from {current} to {latest}");
        return Ok(());
    }

    fs::write(&cargo_toml_path, updated)
        .with_context(|| format!("failed to write {}", cargo_toml_path.display()))?;

    if !args.skip_lockfile {
        process::run(
            &crate::process::Program::new("cargo"),
            &paths.rust_dir,
            &[
                "update".into(),
                "-p".into(),
                "godot".into(),
                "--precise".into(),
                latest,
            ],
        )?;
    }

    Ok(())
}

fn is_sha(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_https_and_ssh_github_urls() {
        assert_eq!(
            github_api_commit_url("https://github.com/godot-rust/gdext.git", "master"),
            Some("https://api.github.com/repos/godot-rust/gdext/commits/master".to_owned())
        );
        assert_eq!(
            github_api_commit_url("git@github.com:godot-rust/gdext.git", "main"),
            Some("https://api.github.com/repos/godot-rust/gdext/commits/main".to_owned())
        );
    }

    #[test]
    fn replaces_godot_inline_table_rev() {
        let input = "[dependencies]\ngodot = { git = \"https://github.com/godot-rust/gdext\", rev = \"1111111111111111111111111111111111111111\" }\n";
        let output = set_godot_rev(input, "2222222222222222222222222222222222222222").unwrap();
        assert!(output.contains("rev = \"2222222222222222222222222222222222222222\""));
        assert_eq!(
            read_godot_rev(&output).unwrap(),
            "2222222222222222222222222222222222222222"
        );
    }
}
