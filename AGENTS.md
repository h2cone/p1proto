# Repository Guidelines

## Project Structure & Module Organization

- `godot/`: Godot 4.x project (scenes, assets, plugins). Entry scene: `godot/ui/main_menu.tscn`.
- `rust/`: Rust `cdylib` GDExtension (gameplay logic exposed to Godot via `godot-rust/gdext`).
- `screenshots/`: Media used by `README.md`.
- Generated (do not edit/commit): `godot/.godot/`, `godot/.import/`, `rust/target/`.

## Build, Test, and Development Commands

- Build the extension (debug): `cd rust && cargo build` (outputs a platform `.dll/.so/.dylib` loaded via `godot/.gdextension`).
- Build the extension (release): `cd rust && cargo build --release`.
- Run locally: open `godot/project.godot` in Godot and press F5, or `godot --path godot`.
- Format/lint Rust: `cd rust && cargo fmt` and `cd rust && cargo clippy -- -D warnings`.

## Coding Style & Naming Conventions

- Rust: follow `rustfmt` defaults; `snake_case` for modules/functions, `PascalCase` for types, `SCREAMING_SNAKE_CASE` for constants.
- Godot: prefer Godot’s default formatting for GDScript; keep script names `snake_case.gd` and scenes `snake_case.tscn` (see `godot/entity/` and `godot/ui/`).
- Generated level content: LDtk importer outputs room scenes as `godot/pipeline/ldtk/levels/Room_{x}_{y}.scn`; treat these as build artifacts.

## Testing Guidelines

- Rust unit tests live alongside modules (e.g., `rust/src/save/`, `rust/src/rooms/`); run `cd rust && cargo test`.
- Keep logic testable by isolating pure functions/structs from Godot node glue; validate gameplay changes by running the project in Godot.

## Commit & Pull Request Guidelines

- Commit messages are short and imperative; the history mixes plain verbs and Conventional Commits (e.g., `feat: ...`, `refactor: ...`, `feat(scope): ...`).
- PRs should include: what changed, how to test (Godot version + build profile), and a screenshot/GIF for visible gameplay/UI/level changes.
