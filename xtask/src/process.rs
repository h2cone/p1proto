use anyhow::{Context, Result};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program {
    path: PathBuf,
}

impl Program {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

pub fn run(program: &Program, cwd: &Path, args: &[String]) -> Result<()> {
    let status = Command::new(program.path())
        .args(args)
        .current_dir(cwd)
        .status()
        .with_context(|| format!("failed to start {}", program.path().display()))?;

    if status.success() {
        Ok(())
    } else {
        anyhow::bail!(
            "{} {} failed with status {}",
            program.path().display(),
            args.join(" "),
            status
        )
    }
}

pub fn capture_stdout(program: &str, cwd: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .with_context(|| format!("failed to start {program}"))?;

    if !output.status.success() {
        anyhow::bail!(
            "{program} {} failed with status {}\nstderr:\n{}",
            args.join(" "),
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn string_args<const N: usize>(args: [&str; N]) -> Vec<String> {
    args.into_iter().map(String::from).collect()
}

pub fn push_os_arg(args: &mut Vec<String>, value: impl AsRef<OsStr>) {
    args.push(value.as_ref().to_string_lossy().into_owned());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn stores_program_path() {
        let program = Program::new("godot");
        assert_eq!(program.path(), Path::new("godot"));
    }
}
