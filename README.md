# p1proto

![Gameplay preview](assets/run.gif)

A compact 2D platformer prototype built with **Godot 4** and a **Rust GDExtension**. Room-to-room traversal, stateful world objects, lightweight save data, and gameplay logic on the Rust side.

## Highlights

- Player movement with coyote time, jump buffering, jump cut, and ground turn acceleration.
- Multi-room traversal via boundary transitions and portal teleports.
- LDtk-authored rooms imported as Godot scenes.
- Stateful entities: checkpoints, collectible stars, keys/locks, pressure plates, moving/crumbling platforms, pushable crates, portals, switch doors.
- Menu flow (New Game / Continue), pause menu, star counter, and explored-room world map.

## Quick Start

Requires **Rust** (`cargo`) and **Godot 4** on `PATH`.

Cross-platform workflow:

```bash
cargo xtask run                  # build debug Rust extension and launch
cargo xtask run --build release  # release build
cargo xtask run --editor         # open Godot editor
cargo xtask export               # create export output
```

## Controls

| Key | Action |
|-----|--------|
| Left / Right | Move |
| Up / Down | Climb ladders; Up activates portals; Down drops through one-way platforms |
| Space | Jump |
| R | Respawn at checkpoint |
| M | Toggle world map |
| Shift | Show room grid while held |
| Esc | Pause |
| B | Toggle background music |

## Repository Layout

| Directory | Contents |
|-----------|----------|
| [`godot/`](godot/) | Godot project, scenes, assets, pipelines, and add-ons |
| [`rust/`](rust/) | GDExtension crate — gameplay, UI, rooms, save logic |
| [`xtask/`](xtask/) | Cross-platform workflow tool invoked through `cargo xtask` |
| [`screenshots/`](screenshots/) | Media for docs and store listings |
| [`docs/`](docs/) | Supporting documentation |

## License

[MIT](LICENSE) — bundled add-on licenses under [`godot/addons/`](godot/addons/) may differ.
