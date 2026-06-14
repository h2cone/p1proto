# Repository Guidelines

## Project Structure & Module Organization
`rust/` contains the Godot GDExtension crate. Core gameplay state lives in `rust/src/core`, runtime orchestration in `rust/src/game`, player behavior in `rust/src/player`, entities in `rust/src/entity`, room loading in `rust/src/rooms`, save logic in `rust/src/save`, and Rust-backed UI in `rust/src/ui`.

`godot/` is the Godot 4 project. Scene files live under folders such as `godot/entity/`, `godot/player/`, and `godot/ui/`. Content pipelines are under `godot/pipeline/ldtk` and `godot/pipeline/aseprite`. Third-party add-ons are vendored in `godot/addons/`.

Use `xtask/` for local workflow automation, `docs/` for supporting notes, and `screenshots/` for repo media. Do not edit generated build output in `rust/target/`, `xtask/target/`, or `export/`.

## Build, Test, and Development Commands
- `cargo xtask run` builds the debug Rust extension and launches the Godot project.
- `cargo xtask run --build release` launches Godot against a release Rust build.
- `cargo xtask run --editor` opens the Godot editor instead of the game.
- `cd rust; cargo test --locked` runs the Rust unit tests.
- `cd rust; cargo fmt --check` verifies Rust formatting.
- `cargo test --manifest-path xtask/Cargo.toml --locked` runs the Rust workflow tool tests.
- `cargo xtask export` creates an export in `export/`.

## Coding Style & Naming Conventions
Rust follows standard `rustfmt` output: 4-space indentation, `snake_case` functions/modules, and `PascalCase` types. Keep gameplay logic in Rust and use Godot scenes/resources as data and wiring. Match existing asset names such as `collectible_star.tscn` and `world_map_model.rs`. Preserve the current tab-indented style in existing GDScript files.

## Testing Guidelines
Tests live inline with Rust modules under `#[cfg(test)]`; there is no separate integration test suite yet. Add or update unit tests when changing movement, persistence, room transitions, or other deterministic logic. Prefer tests that avoid needing a live Godot runtime.

## Commit & Pull Request Guidelines
Recent history uses Conventional Commits, often with scopes: `feat(player): ...`, `fix: ...`, `refactor(save): ...`, `chore: ...`. Keep commits small and imperative. PRs should summarize gameplay impact, list verification steps, link related issues, and include screenshots or GIFs for UI, scene, or level changes.

## Configuration Tips
Keep both `cargo` and `godot` on `PATH`. When updating `godot-rust`, prefer `cargo xtask update-gdext` so `rust/Cargo.toml` and `rust/Cargo.lock` stay in sync.
