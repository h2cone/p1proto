mod cli;
mod export;
mod godot;
mod ldtk_resize;
mod paths;
mod process;
mod run;
mod update_gdext;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let paths = paths::ProjectPaths::discover()?;

    match cli.command {
        Command::Run(args) => run::execute(&paths, args),
        Command::Export(args) => export::execute(&paths, args),
        Command::UpdateGdext(args) => update_gdext::execute(&paths, args),
        Command::ResizeLdtkRooms(args) => ldtk_resize::execute(&paths, args),
    }
}
