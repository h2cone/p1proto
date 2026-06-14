mod cli;
mod godot;
mod paths;
mod process;
mod run;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let paths = paths::ProjectPaths::discover()?;

    match cli.command {
        Command::Run(args) => run::execute(&paths, args),
        Command::Export(args) => {
            println!("export in {}: {args:?}", paths.repo_root.display());
            Ok(())
        }
        Command::UpdateGdext(args) => {
            println!("update-gdext in {}: {args:?}", paths.repo_root.display());
            Ok(())
        }
        Command::ResizeLdtkRooms(args) => {
            println!("resize-ldtk-rooms in {}: {args:?}", paths.repo_root.display());
            Ok(())
        }
    }
}
