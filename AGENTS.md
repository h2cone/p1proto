# Repository Guidelines

## Project Structure & Module Organization

- `godot/`: Godot 4.5 project (`project.godot`), scenes and assets.
  - `ui/`: menus and HUD scenes (entry point is `ui/main_menu.tscn`).
  - `entity/`, `player/`: gameplay scenes used by levels.
  - `pipeline/ldtk/`: LDtk source + generated room scenes (`levels/Room_{x}_{y}.scn`).
  - `.godot/`: editor cache (do not commit).
- `rust/`: Rust GDExtension (`cdylib`) loaded by `godot/rust.gdextension`.
  - `src/player/`, `src/entity/`, `src/rooms/`, `src/save/`, `src/ui/`, `src/game/`
- `screenshots/`: gifs/images referenced by docs (e.g. `screenshots/run.gif`).

## Build, Test, and Development Commands

Run from the repo root:

```bash
cd rust
cargo build            # build debug extension
cargo build --release  # build optimized extension
cargo test             # run Rust unit tests
cargo test save::      # run save-system tests only
```

Run the game:

```bash
cd godot
godot --path .         # or open `godot/` in the Godot editor and press F5
```

## Coding Style & Naming Conventions

- Rust: format with `cargo fmt` (rustfmt; 4-space indentation); use `snake_case` for modules/files and `PascalCase` for types.
- Godot: keep `.tscn`/`.gd` in `snake_case`; prefer placing new gameplay scenes under `godot/entity/` and UI under `godot/ui/`.
- Keep Rust↔Godot boundaries explicit: Rust classes exposed to Godot live under `rust/src/` and are loaded via `godot/rust.gdextension`.

## Testing Guidelines

- Tests use Rust’s built-in test harness (`#[test]`) and live next to the code (common areas: `rust/src/save/*`, `rust/src/rooms/*`).
- Prefer small, deterministic unit tests over integration-style tests for gameplay logic.
- No formal coverage target; add tests for bugfixes and new logic in `save/` and `rooms/` where feasible.

## Commit & Pull Request Guidelines

- Commit messages generally follow Conventional Commits: `feat: ...`, `fix: ...`, `refactor: ...`, `docs: ...`, `chore: ...` (optional scope like `feat(glicol): ...`).
- PRs should include: a brief summary, run instructions, and updated media in `screenshots/` for visible changes.

## Notes for Contributors

- This repo’s `.gitignore` ignores `AGENTS.md`/`CLAUDE.md`/`GEMINI.md`. Remove those entries if you want this guide committed.
