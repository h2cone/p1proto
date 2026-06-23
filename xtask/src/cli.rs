use clap::{Args, Parser, Subcommand, ValueEnum};

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
    UpdateGodotAddons(UpdateGodotAddonsArgs),
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum GodotAddonSelection {
    All,
    Ldtk,
    Aseprite,
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
pub struct UpdateGodotAddonsArgs {
    #[arg(long, value_enum, default_value_t = GodotAddonSelection::All)]
    pub addon: GodotAddonSelection,
    #[arg(long = "ref")]
    pub ref_name: Option<String>,
    #[arg(long)]
    pub dry_run: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_run_release_editor() {
        let cli = Cli::try_parse_from(["xtask", "run", "--build", "release", "--editor"]).unwrap();
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

    #[test]
    fn resize_ldtk_rooms_is_not_registered() {
        let error = Cli::try_parse_from(["xtask", "resize-ldtk-rooms"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::InvalidSubcommand);
    }

    #[test]
    fn parses_update_godot_addons_single_ref() {
        let cli = Cli::try_parse_from([
            "xtask",
            "update-godot-addons",
            "--addon",
            "aseprite",
            "--ref",
            "v9.8.0",
            "--dry-run",
        ])
        .unwrap();

        assert!(matches!(
            cli.command,
            Command::UpdateGodotAddons(UpdateGodotAddonsArgs {
                addon: GodotAddonSelection::Aseprite,
                ref_name: Some(_),
                dry_run: true,
            })
        ));
    }
}
