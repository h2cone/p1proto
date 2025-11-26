# Gemini Context: p1proto

## Project Overview
`p1proto` is a 2D game project built with **Godot 4.5+** and **Rust** via GDExtension (`godot-rust/gdext`).
The project uses a hybrid workflow:
- **Core Logic:** Implemented in Rust (`rust/` directory) for performance and safety.
- **Game Design:** Scenes, levels, and UI are assembled in the Godot Editor (`godot/` directory).
- **Assets:** 2D pixel art created in Aseprite (`aseprite/` directory).

## Directory Structure
- `rust/`: Rust source code. Defines custom Nodes and game systems.
    - `src/`: Contains logic for `Player`, `Level`, `World`, etc.
    - `Cargo.toml`: Dependencies (mainly `godot`).
- `godot/`: The Godot project root.
    - `addons/`: Plugins (e.g., `AsepriteWizard`).
    - `graphics/`: Exported assets (PNGs, resources).
    - `*.tscn`: Scene files (e.g., `game.tscn`, `player.tscn`).
- `aseprite/`: Source `.aseprite` files and export scripts.

## Development Environment

### Prerequisites
- **Rust Toolchain:** 2024 edition or later.
- **Godot Engine:** Version 4.5+ (referenced as `godot4` in commands).
- **Aseprite:** (Optional) For editing source art.

### Build & Run Commands
The project follows specific workflows defined in `AGENTS.md`.

| Task | Command | Context |
| :--- | :--- | :--- |
| **Check Rust Code** | `cd rust && cargo check` | Validate syntax/types |
| **Build Extension** | `cd rust && cargo build` (or `--release`) | Compile `.dll` / `.so` for Godot |
| **Test Rust** | `cd rust && cargo test --all` | Run unit tests |
| **Run Editor** | `cd godot && godot4 --editor` | Open Godot Editor |
| **Run Game (CI)** | `cd godot && godot4 --headless --run res://game.tscn` | Headless smoke test |
| **Format Code** | `cargo fmt` | Apply Rust formatting |
| **Lint Code** | `cargo clippy --workspace` | Check for Rust patterns |

## Architectural Patterns

### GDExtension (Rust <-> Godot)
- Custom nodes are defined in Rust (e.g., `Player` node in `player.tscn`).
- Rust structs implement `GodotClass` and inherit from engine classes (e.g., `CharacterBody2D`, `Node`).
- The extension is registered in `rust/src/lib.rs`.

### Asset Pipeline
1.  Create/Edit art in `aseprite/`.
2.  Export sprite sheets/animations to `godot/graphics/`.
3.  Use `.import` files in Godot to manage texture settings (e.g., nearest-neighbor filtering).

## Conventions
*   **Rust:** `snake_case` for modules/functions, `PascalCase` for types. 4-space indent.
*   **Godot:** `snake_case` for node names (`player_body`), `PascalCase` (Noun-based) for scene files (`player.tscn`).
*   **Git:** Imperative commit messages ("Add player movement"). One feature per commit.
