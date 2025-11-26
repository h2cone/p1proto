# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a 2D game project built with Godot 4.5+ and Rust via GDExtension (godot-rust/gdext). The architecture follows a hybrid approach:
- Core game logic is implemented in Rust for performance and type safety
- Scene composition, level design, and editor workflows use Godot
- Pixel art assets are created in Aseprite and exported to the Godot project

## Directory Structure

- `rust/` - Rust GDExtension library that compiles to a dynamic library loaded by Godot
  - `src/lib.rs` - Extension entry point that registers all custom classes
  - `src/player.rs` - Player CharacterBody2D implementation
  - `src/player_movement.rs` - Movement state machine and physics logic
  - `src/game.rs`, `src/world.rs`, `src/level.rs` - Game structure nodes
  - `Cargo.toml` - Uses Rust 2024 edition, depends on godot-rust/gdext from git

- `godot/` - Godot project root (open this folder in Godot editor)
  - `.gdextension` - Configures the Rust library paths for different platforms
  - `game.tscn` - Main game scene
  - `player.tscn` - Player scene using the Rust Player class
  - `graphics/` - Exported sprite sheets and textures
  - `addons/AsepriteWizard/` - Plugin for importing Aseprite files

- `aseprite/` - Source artwork and export scripts

## Build Commands

### Rust Development
```bash
cd rust
cargo check              # Fast syntax/type validation
cargo build              # Debug build (creates rust/target/debug/rust.dll or librust.so)
cargo build --release    # Optimized build (creates rust/target/release/)
cargo test --all         # Run unit tests
cargo fmt                # Format code
cargo clippy --workspace # Run linter
```

### Godot Development
```bash
cd godot
godot4 --editor                      # Open Godot editor
godot4 --headless --run res://game.tscn  # Run game headlessly (for CI)
```

### Development Workflow
1. Make Rust changes in `rust/src/`
2. Run `cd rust && cargo build` to compile the extension
3. Open or reload Godot editor to pick up changes (reloadable = true in .gdextension)
4. Test in Godot editor or run the game scene

## Architecture Patterns

### GDExtension Integration
- Rust structs derive `#[derive(GodotClass)]` and specify their base class with `#[class(base=Node)]`
- Implement trait `INode`, `ICharacterBody2D`, etc. to override Godot lifecycle methods
- Use `godot::prelude::*` for core types (Vector2, Node, etc.)
- Register classes by adding modules to `lib.rs` - the #[gdextension] macro handles registration
- Access base class methods via `self.base()` (immutable) or `self.base_mut()` (mutable)

### Player Movement Architecture
The player movement system is split into two files:
- `player.rs` - CharacterBody2D wrapper that owns the movement controller
- `player_movement.rs` - Pure logic state machine with MovementState enum (Air/Floor)

Movement logic is factored into a separate struct to:
- Enable unit testing without Godot dependencies
- Keep physics logic decoupled from Godot API calls
- Allow reuse across different character types

The pattern: Extract game logic into plain Rust structs, then call them from GDExtension classes.

### Input Actions
The project uses Godot's input action system. Actions referenced in code:
- `walk_left` / `walk_right` - Horizontal movement
- `jump` - Jump action

These must be configured in Godot's Project Settings > Input Map.

### Physics Configuration
Gravity is pulled from ProjectSettings in `player.rs:ready()`:
```rust
let gravity = settings.get("physics/2d/default_gravity").to::<f64>() as f32;
```

Movement constants (walk_speed, jump_velocity, etc.) are hardcoded in MovementConfig initialization in `player.rs:ready()`.

## Coding Conventions

### Rust
- Follow rustfmt defaults (4-space indent, 120 char line width)
- Use `snake_case` for functions, modules, and variables
- Use `PascalCase` for types and structs
- Module names match file names (`player_movement.rs` → `mod player_movement`)
- Add unit tests in `#[cfg(test)]` blocks within the same file
- For cross-module integration tests, use `rust/tests/`

### Godot/GDScript
- Use `snake_case` for node names in scenes (e.g., `player_body`)
- Scene files use noun-based PascalCase names (e.g., `player.tscn`)
- Signals use camelCase (Godot convention)

### Asset Naming
- Use lowercase-dash format for exported assets (e.g., `jelly-idle.png`)
- Keep `.aseprite` source files in sync with exports
- Document export settings in Lua scripts when they change

## Git Workflow

### Commit Style
- Use imperative mood ("Add player movement" not "Added player movement")
- One feature or fix per commit for easy bisecting
- Examples from history:
  - "Add Aseprite integration and jelly character sprite"
  - "Add player character with movement system"

### Pull Request Guidelines
- Describe gameplay impact and affected scenes/scripts
- Include reproduction steps or screenshots for visual changes
- Run `cargo test --all` before submitting
- Attach lint/test output for non-trivial changes

### What to Commit
- Source files: `*.rs`, `*.tscn`, `*.gd`, `Cargo.toml`
- Asset sources: `*.aseprite`, export scripts
- Exported assets: `*.png` in `godot/graphics/`
- Configuration: `.gdextension`, `project.godot`

### What NOT to Commit
- `rust/target/` - Build artifacts (gitignored)
- `godot/.godot/` - Editor cache (gitignored)
- Editor-only temporary nodes in scenes
- Unstable editor metadata

## Important Notes

### Rust Edition 2024
The project uses Rust edition 2024 in Cargo.toml. Ensure your toolchain supports this.

### Build Optimization
The `[profile.dev]` and `[profile.dev.package."*"]` settings in Cargo.toml set opt-level=1 to speed up debug builds while keeping reasonable performance for game testing.

### Cross-Platform Libraries
The `.gdextension` file configures library paths for Linux (.so), Windows (.dll), and macOS (.dylib). When building for release, ensure you're targeting the correct platform.

### AsepriteWizard Plugin
The `addons/AsepriteWizard/` plugin enables direct import of `.aseprite` files into Godot. Keep source files in `aseprite/` and export to `godot/graphics/` for version control clarity.
