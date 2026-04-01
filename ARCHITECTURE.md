# Architecture

## Bird's Eye View

`p1proto` is a Godot 4 platformer prototype whose runtime behavior is implemented primarily in a Rust GDExtension. Godot owns scene composition, editor configuration, assets, and import tooling; Rust owns movement rules, room/session orchestration, persistent world state, and most menu/HUD behavior.

At runtime, Godot instantiates Rust `GodotClass` nodes from scene files such as `game.tscn`, `player/player.tscn`, and `ui/*.tscn`. The active gameplay loop runs through `GameRoomManager`: it resolves the starting spawn, loads the current LDtk-derived room scene, spawns the player, and then drives boundary transitions, portal teleports, death handling, and explored-room tracking. Room content is authored in LDtk, post-processed into `pipeline/ldtk/levels/Room_<x>_<y>.scn`, and populated with entity scenes whose scripts are also Rust classes.

## Code Map

### `rust/`

- `src/core` — Pure gameplay/session/progress logic. Key files: `player.rs`, `world.rs`, `session.rs`, `progress.rs`. Relationships: consumed by `game`, `player`, `entity`, and `save`; it defines movement rules, room-transition planning, spawn resolution, and the in-memory progress repository.
- `src/game` — Top-level gameplay orchestration and room lifecycle. Key files: `mod.rs` (`Game`), `room_manager.rs` (`GameRoomManager`), `room_runtime.rs`, `portal_connector.rs`, `player_spawner.rs`. Relationships: depends on `core`, `rooms`, `player`, `entity`, and `save`; it is the owner of the live player/room session.
- `src/player` — The player-facing Godot class and adapters around core movement. Key files: `mod.rs` (`Player`), `input_adapter.rs`, `animation.rs`, `platform.rs`, `push.rs`, `hazard.rs`. Relationships: wraps `core::player::PlayerMovement` with Godot physics, animation, and collision handling; instantiated by `player/player.tscn`.
- `src/entity` — Rust implementations of room entities. Key files: `checkpoint.rs`, `collectible_star.rs`, `plain_key.rs`, `plain_lock.rs`, `portal.rs`, `pressure_plate.rs`, `switch_door.rs`, `moving_platform.rs`, `crumbling_platform.rs`, `pushable_crate.rs`, `persistence.rs`. Relationships: mostly leaf behaviors instantiated inside imported room scenes; persistent entities flow through `core::progress` via `entity::persistence`; `game` reaches in mainly for portal signal hookup.
- `src/rooms` — Room-scene loading and caching. Key files: `loader.rs` (`RoomLoader`). Relationships: used by `game::room_runtime`; depends only on Godot resource loading and the room naming convention.
- `src/save` — Thin facade over progress state. Key files: `mod.rs`, `SaveApi`. Relationships: re-exports and wraps `core::progress` for Rust UI and any Godot-facing callers; used by `game` and `ui`.
- `src/ui` — Rust-backed menus and HUD/map widgets. Key files: `main_menu.rs`, `pause_menu.rs`, `star_counter.rs`, `world_map.rs`, `world_map_model.rs`. Relationships: depends on `save`, and `world_map.rs` also reads `GameRoomManager` to highlight the current room.

### `godot/`

- `project.godot`, `rust.gdextension`, `game.tscn` — Project entry points and extension wiring. Key files: `project.godot`, `rust.gdextension`, `game.tscn`. Relationships: `project.godot` points the app at `ui/main_menu.tscn`; `rust.gdextension` loads the compiled Rust library; `game.tscn` composes the Rust runtime nodes (`Game`, `GameRoomManager`, pause menu, world map, star counter).
- `player/`, `entity/`, `ui/` — Scene shells and exported data for Rust classes. Key files: `player/player.tscn`, `entity/*.tscn`, `ui/main_menu.tscn`, `ui/pause_menu.tscn`, `ui/world_map.tscn`, `ui/star_counter.tscn`. Relationships: these scenes provide the node tree, collision shapes, sprite resources, and exported fields that the Rust classes expect.
- `pipeline/ldtk` — Level-authoring pipeline. Key files: `tilemap.ldtk`, `level_post_import.gd`, `entities_post_import.gd`, `regenerate_rooms.gd`, `levels/Room_*.scn`. Relationships: LDtk is the source of truth for room topology and entity placement; post-import scripts instantiate entity scenes, stamp metadata such as `ldtk_iid`, and generate the room scenes that `RoomLoader` expects.
- `pipeline/aseprite` and `pipeline/glicol` — Asset source pipelines. Key files: `aseprite/src/*.aseprite`, `aseprite/wizard/*.res`, `glicol/bgm.glicol`, `glicol/bgm.ogg`. Relationships: produce sprite frames and audio resources referenced by the scene shells; they feed runtime presentation but are not part of gameplay control flow.
- `addons/` — Vendored editor/import plugins. Key files: `addons/ldtk-importer/*`, `addons/AsepriteWizard/*`. Relationships: used by the content pipeline and editor workflow; not part of the game's own module graph.

### Repository Support

- `scripts/` — Local build/export/update automation. Key files: `run.ps1`, `export.ps1`, `update_gdext.ps1`. Relationships: coordinate `cargo` and Godot, but do not participate in runtime behavior.

## Architectural Invariants

- `rust/src/core` is the decision-making layer for movement, room/session planning, and progress tracking. Higher layers may depend on it; it does not depend on `game`, `player`, `entity`, or `ui`.
- `GameRoomManager` owns the active gameplay session. Room loads/unloads, boundary transitions, portal teleports, player spawning, and death reload/restart decisions all funnel through it.
- Room traversal is coordinate-based, not graph-authored in code. Adjacent room transitions come from `BoundaryDetector` plus room existence checks; portals are the explicit non-adjacent transition path.
- Imported rooms must keep the `Room_<x>_<y>.scn` naming scheme and an `Entities` layer. `RoomLoader`, `GameRoomManager`, and `portal_connector` assume that structure.
- Persistent world state goes through `core::progress`, usually via `entity::persistence`, with LDtk IID metadata preferred over position-based fallback keys. Individual entities do not maintain their own save stores.
- Scene files are mostly composition and data. Runtime gameplay logic for player, rooms, menus, HUD, and entities lives in Rust `GodotClass` implementations; GDScript is mainly reserved for import/editor tooling.
- `rooms::RoomLoader` only knows how to load and cache scenes. It intentionally does not know about save state, player setup, or transition policy.

## Cross-Cutting Concerns

- Persistence is currently process-local. `core::progress` stores checkpoints, collected entities, star count, and explored rooms in a thread-local repository so state survives scene changes and menu transitions, but there is no on-disk save/load layer yet.
- Content generation is a first-class part of the architecture. LDtk post-import scripts assign exported fields and metadata that runtime Rust code depends on, especially for room coordinates, portal destinations, pressure-plate targets, and persistent entity IDs.
- Runtime communication leans on Godot signals at module seams: menu buttons change scenes, portals request teleports, the player emits `death_finished`, and entities emit state-change signals while delegating shared state to `save`/`progress`.
- Testing is mostly inline Rust unit tests. The repo favors keeping deterministic logic in `core`, `save`, `rooms`, and small helper modules so it can be tested without a live Godot runtime.
