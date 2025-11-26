# Repository Guidelines

## Project Structure & Module Organization
- `rust/` contains the gameplay crate (`src/player.rs`, `src/player_movement.rs`, etc.) plus `Cargo.toml` for dependency management.
- `godot/` hosts scenes (`game.tscn`, `player.tscn`), editor scripts, and engine config (`project.godot`). Open this folder as the Godot project root.
- `aseprite/` stores production art and scripts like `create_jelly_character.lua`. Export sprite sheets into `godot/graphics/` to keep runtime assets versioned.

## Build, Test, and Development Commands
- `cd rust && cargo check` – fast validation of the Rust crate used by the Godot extension.
- `cd rust && cargo build --release` – produces optimized libraries that Godot loads via `.gdextension`.
- `cd godot && godot4 --editor` – open the project for scene wiring; `godot4 --headless --run res://game.tscn` runs the level for CI smoke tests.
- `cargo fmt` and `cargo clippy --workspace` keep Rust formatting and lints consistent before committing.

## Coding Style & Naming Conventions
- Use rustfmt defaults (4-space indent, `snake_case` functions/modules, `PascalCase` types). Mirror module names in filenames (`player_movement.rs`).
- GDScript files stay camelCase for signals but prefer snake_case for nodes (`player_body`). Scenes follow noun-based filenames (`player.tscn`).
- Art assets adopt lowercase-dash names (`jelly-idle.png`) so exports match Godot resource expectations.

## Testing Guidelines
- Add Rust unit tests alongside modules using `#[cfg(test)]` blocks; integration tests go under `rust/tests/` when systems span modules.
- Run `cargo test --all` before publishing a PR; fail-fast fixes are easier than debugging through Godot.
- For gameplay regressions, capture short Godot recordings or GIFs from the `godot/` editor describing the scenario tested.

## Commit & Pull Request Guidelines
- Follow the existing imperative, concise style (`Add player character with movement system`). One feature or fix per commit keeps bisects easy.
- Each PR should describe gameplay impact, affected scenes/scripts, and include reproduction steps or relevant screenshots. Link tracking issues or TODO IDs from scene files when available.
- CI reviewers expect lint/test logs (`cargo test`, `godot4 --headless`) pasted or attached when the change is non-trivial.

## Asset & Editor Tips
- Keep `.aseprite` sources updated when exporting new sprites; document export settings in the Lua helper when they change.
- Godot `.godot` metadata regenerates often—only commit stable changes (scenes, scripts, config). Clean up editor-only nodes before submitting.
