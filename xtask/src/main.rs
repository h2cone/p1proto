mod cli;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Run(args) => {
            println!("run: {args:?}");
        }
        Command::Export(args) => {
            println!("export: {args:?}");
        }
        Command::UpdateGdext(args) => {
            println!("update-gdext: {args:?}");
        }
        Command::ResizeLdtkRooms(args) => {
            println!("resize-ldtk-rooms: {args:?}");
        }
    }

    Ok(())
}
