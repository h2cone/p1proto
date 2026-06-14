use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "xtask", about = "Cross-platform project workflow tasks")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Run(RunArgs),
    Export(ExportArgs),
    UpdateGdext(UpdateGdextArgs),
    ResizeLdtkRooms(ResizeLdtkRoomsArgs),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum BuildMode {
    Debug,
    Release,
    Both,
    None,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum ExportTarget {
    Windows,
    Macos,
}

#[derive(Debug, Args)]
pub struct RunArgs {
    #[arg(long, value_enum, default_value_t = BuildMode::Debug)]
    pub build: BuildMode,
    #[arg(long, default_value = "godot")]
    pub godot_exe: String,
    #[arg(long)]
    pub editor: bool,
    #[arg(long)]
    pub headless: bool,
    #[arg(last = true)]
    pub godot_args: Vec<String>,
}

#[derive(Debug, Args)]
pub struct ExportArgs {
    #[arg(long, value_enum, default_value_t = ExportTarget::Windows)]
    pub target: ExportTarget,
    #[arg(long, default_value = "godot")]
    pub godot_exe: String,
    #[arg(long, default_value = "Windows Desktop")]
    pub preset_name: String,
    #[arg(long)]
    pub force_create_export_preset: bool,
    #[arg(long)]
    pub skip_debug_build: bool,
}

#[derive(Debug, Args)]
pub struct UpdateGdextArgs {
    #[arg(long, default_value = "https://github.com/godot-rust/gdext.git")]
    pub repo_url: String,
    #[arg(long, default_value = "master")]
    pub branch: String,
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long)]
    pub skip_lockfile: bool,
}

#[derive(Debug, Args)]
pub struct ResizeLdtkRoomsArgs {
    #[arg(long, default_value = "godot/pipeline/ldtk/tilemap.ldtk")]
    pub path: PathBuf,
    #[arg(long, default_value_t = 480)]
    pub width: i64,
    #[arg(long, default_value_t = 360)]
    pub height: i64,
    #[arg(long)]
    pub insert_x: Option<i64>,
    #[arg(long)]
    pub insert_y: Option<i64>,
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long)]
    pub report_directory: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_run_release_editor() {
        let cli =
            Cli::try_parse_from(["xtask", "run", "--build", "release", "--editor"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Run(RunArgs {
                build: BuildMode::Release,
                editor: true,
                headless: false,
                ..
            })
        ));
    }
}
