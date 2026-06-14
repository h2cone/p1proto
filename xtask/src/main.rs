mod cli;
mod godot;
mod paths;
mod process;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let paths = paths::ProjectPaths::discover()?;

    match cli.command {
        Command::Run(args) => {
            println!("run in {}: {args:?}", paths.repo_root.display());
        }
        Command::Export(args) => {
            println!("export in {}: {args:?}", paths.repo_root.display());
        }
        Command::UpdateGdext(args) => {
            println!("update-gdext in {}: {args:?}", paths.repo_root.display());
        }
        Command::ResizeLdtkRooms(args) => {
            println!("resize-ldtk-rooms in {}: {args:?}", paths.repo_root.display());
        }
    }

    Ok(())
}
