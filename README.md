# p1proto

![Gameplay preview](screenshots/run.gif)

`p1proto` is a compact 2D platformer prototype built with Godot 4.6 and a Rust GDExtension. The project is set up as a reference sample for room-to-room traversal, stateful world objects, lightweight save data, and a Godot + Rust workflow that keeps gameplay logic on the Rust side.

## Highlights

- Player movement with coyote time, jump buffering, jump cut, and faster ground turn acceleration.
- Multi-room traversal with boundary transitions and portal-based teleports.
- LDtk-authored rooms imported as Godot scenes.
- Stateful entities including checkpoints, collectible stars, keys and locks, pressure plates, moving platforms, crumbling platforms, pushable crates, portals, and switch doors.
- Menu flow with New Game / Continue, in-game pause menu, star counter, and an explored-room world map.
- Small Rust unit test suite covering movement, save logic, and room transition helpers.

## Tech Stack

- Godot 4.6 project in [`godot/`](godot/)
- Rust GDExtension crate in [`rust/`](rust/)
- LDtk pipeline assets in [`godot/pipeline/ldtk/`](godot/pipeline/ldtk/)
- Aseprite source and generated sprite assets in [`godot/pipeline/aseprite/`](godot/pipeline/aseprite/)

## Prerequisites

- Rust toolchain with `cargo`
- Godot 4.6 available as `godot` on `PATH`, or passed explicitly to the helper scripts
- Windows PowerShell if you want to use the repo scripts under [`scripts/`](scripts/)

## Quick Start

Build the Rust extension, then launch the Godot project:

```bash
cd rust
cargo build

cd ../godot
godot --path .
```

The project expects the GDExtension library to exist under `rust/target/...`, so opening `godot/` before building `rust/` will not work on a clean checkout.

## Windows Helper Scripts

Run the game from the repo root:

```powershell
./scripts/run.ps1
```

Useful variants:

```powershell
./scripts/run.ps1 -Build Release
./scripts/run.ps1 -Editor
./scripts/run.ps1 -Build None
```

Export a Windows build:

```powershell
./scripts/export.ps1
```

Notes:

- `export.ps1` expects Godot export templates to be installed.
- The export script copies `rust.dll` next to the exported executable.

## Controls

- Left / Right: move
- Space: jump
- Up: activate portals
- M: toggle world map
- Esc: pause
- B: toggle background music

## Development

Build and test from the repo root:

```bash
cd rust
cargo build
cargo build --release
cargo test
cargo test save::
```

Refresh the pinned `gdext` revision:

```powershell
./scripts/update_gdext.ps1
```

## Repository Layout

- [`godot/`](godot/): Godot project, scenes, assets, import pipelines, export presets, and add-ons
- [`rust/`](rust/): Rust GDExtension crate that implements gameplay, UI, rooms, and save logic
- [`scripts/`](scripts/): local helper scripts for running, exporting, and updating the `gdext` dependency
- [`screenshots/`](screenshots/): media used in documentation and store listings
- [`docs/`](docs/): supporting project documentation

## Notes

- This repo is a playable prototype / demo project, not an editor plugin.
- Third-party Godot add-ons are vendored under [`godot/addons/`](godot/addons/) to support the content pipeline.
- The root license is MIT. Review bundled add-on licenses separately if you redistribute the full project.

## License

[MIT](LICENSE)
