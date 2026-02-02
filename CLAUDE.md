# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a 2D platformer game prototype built with Godot 4.5 and Rust using gdext (godot-rust bindings). The project follows a hybrid architecture where game logic is implemented in Rust and compiled to a native library, while Godot handles rendering, scene management, and level data.

## Build and Development Commands

### Building the Rust Extension
```bash
cd rust
cargo build              # Debug build
cargo build --release    # Release build
cargo test               # Run all unit tests
cargo test save::       # Run only save system tests
```

The Rust code compiles to a dynamic library (`.dll` on Windows, `.so` on Linux, `.dylib` on macOS) that Godot loads via the GDExtension system defined in `godot/.gdextension`. Debug builds have optimization level 1 enabled for faster iteration.

### Running the Game
Open the `godot/` directory in Godot 4.5 and press F5, or use the Godot CLI:
```bash
cd godot
godot --path .
```

The entry point is `godot/ui/main_menu.tscn` (configured in `godot/project.godot`). When running from the editor, Rust changes hot-reload automatically (`.gdextension` has `reloadable = true`).

### Testing

Unit tests are located in `#[cfg(test)]` modules within Rust source files, primarily in the save system:

```bash
cd rust
cargo test                    # Run all tests with output
cargo test -- --nocapture    # Run tests with println! output visible
cargo test save::core::tests  # Run specific test module
```

Key test modules:
- `rust/src/save/core.rs::tests` - Save slot system and checkpoint save/load
- `rust/src/save/entity_state.rs::tests` - Entity persistence (keys, locks, stars)
- `rust/src/save/exploration.rs::tests` - Exploration state tracking
- `rust/src/rooms/loader.rs::tests` - Room loading and scene caching

## Architecture

### Godot-Rust Bridge (GDExtension)

The project uses gdext 0.4.3 (tracking git main) to expose Rust classes to Godot:

- **Entry Point**: `rust/src/lib.rs` defines `MyExtension` which implements `ExtensionLibrary`
- **Class Registration**: Rust structs decorated with `#[derive(GodotClass)]` become available in Godot
- **Inheritance**: `#[class(base=SomeGodotClass)]` specifies the Godot base class
- **Trait Implementation**: `INode`, `ICharacterBody2D`, etc. provide lifecycle methods (`init`, `ready`, `physics_process`)
- **Node References**: `OnReady<Gd<T>>` pattern defers node lookup until `ready()` is called, e.g., `OnReady::from_node("AnimatedSprite2D")`

### Module Organization

**`rust/src/player/`** - Player character logic
- `movement.rs`: Movement state machine with `MovementState` enum (Air/Floor), input handling, and physics
- `animation.rs`: Sprite animation mapping (idle, walk, jump, fall, death)
- `input_adapter.rs`: Input handling from Godot input actions
- `mod.rs`: `Player` GodotClass integrating movement, animation, physics, death/hazard detection, and rigid body pushing

**`rust/src/rooms/`** - Room loading and transition system for grid-based levels
- `loader.rs`: `RoomLoader` handles PackedScene caching and instantiation using grid coordinates
- `transition.rs`: `BoundaryDetector` for room transitions (implementation details TBD)
- Design: Calculates adjacent rooms from grid coordinates rather than hardcoded connections

**`rust/src/entity/`** - Game entities (checkpoints, platforms, keys, locks, etc.)
- `checkpoint.rs`: `Checkpoint` Area2D that detects player contact and saves progress; restores activation state on load by matching room coordinates and position
- `moving_platform.rs`: `MovingPlatform` AnimatableBody2D that moves along a path with configurable duration and pause time
- `plain_key.rs`: `PlainKey` Area2D collectible that follows the player after pickup; persists collected state via save system
- `plain_lock.rs`: `PlainLock` StaticBody2D that blocks player until unlocked with a collected key; persists unlocked state
- `pressure_plate.rs`: `PressurePlate` Area2D that activates/deactivates based on body presence (non-persistent)
- `crumbling_platform.rs`: `CrumblingPlatform` AnimatableBody2D with state machine (Idle→Shaking→Crumbling→Fallen); configurable shake_delay and respawn_time
- `switch_door.rs`: `SwitchDoor` StaticBody2D controlled by external triggers; state machine (Closed↔Opening↔Open↔Closing) with animation-driven transitions
- `pushable_crate.rs`: `PushableCrate` RigidBody2D that can be pushed by player; supports freeze/unfreeze
- `portal.rs`: `Portal` Area2D for room-to-room teleportation; emits `teleport_requested` signal when player presses up
- `collectible_star.rs`: `CollectibleStar` Area2D collectible for objectives/progression tracking; persists collected state via save system

**`rust/src/save/`** - Save system with thread-local storage
- `mod.rs`: Thread-local save store using `RefCell<SaveStore>` for safe single-threaded access
- `SaveSnapshot`: Stores room coordinates and player position
- `SaveStore`: Tracks checkpoint slots, pending loads, unlocked locks, and collected keys
- `SaveApi`: Godot-facing API for querying/loading save slots
- Design: In-memory storage using `thread_local!` macro (expandable to file persistence later)

**`rust/src/ui/`** - Menu systems and UI screens
- `main_menu.rs`: Main menu UI
- `pause_menu.rs`: Pause menu UI
- `world_map.rs`: World map/level select screen for progression tracking

**`rust/src/game/`** - High-level game management
- `mod.rs`: `Game` node for game state coordination
- `room_manager.rs`: Room management logic

### Level Data (LDtk Integration)

Levels are created in LDtk and imported via the `ldtk-importer` plugin:
- **Source**: `godot/pipeline/ldtk/tilemap.ldtk` - LDtk project file
- **Output**: `godot/pipeline/ldtk/levels/Room_{x}_{y}.scn` - Generated Godot scenes per room
- **Tileset**: `godot/pipeline/ldtk/tilesets/tileset_8px.res` - 8px tile resources
- **Pattern**: Room scenes follow a grid naming convention where `{x}` and `{y}` are grid coordinates
- **Entity Post-Import**: `godot/pipeline/ldtk/entities_post_import.gd` - Script that processes LDtk entities during import

The `RoomLoader` uses this pattern to dynamically load rooms: `"res://pipeline/ldtk/levels/Room_{x}_{y}.scn"`

#### LDtk Entity System

The post-import script (`entities_post_import.gd`) automatically instantiates scene files for LDtk entities:
- **Entity scenes**: Located at `res://entity/{entity_key}.tscn` where `entity_key` is the lowercase LDtk identifier
- **Room-aware entities**: `checkpoint`, `plain_key`, `plain_lock` automatically receive `room_coords` export from level name
- **MovingPlatform**: Special handling with fields: `travel_x`, `travel_y`, `duration`, `pause_time`

Entities are identified by their LDtk identifier and positioned at anchor point (center by default).

### Input Actions

Defined in `godot/project.godot`:
- `act_walk_left` - Left arrow key
- `act_walk_right` - Right arrow key
- `act_jump` - Spacebar
- `act_down` - Down arrow key
- `act_up` - Up arrow key (portal activation)
- `ui_esc` - Escape key

### Physics Layers

Defined in `godot/project.godot`:
- Layer 1: `player` - Player character collision
- Layer 2: `tile` - Tilemap collision
- Layer 3: `moving_platform` - Moving platform collision
- Layer 4: `plain_lock` - Lock collision (blocks player)
- Layer 5: `plain_key` - Key collision (collectible)
- Layer 13: `crumbling_platform` - Crumbling platform collision

### Display Configuration

- Base viewport: 320x240 (pixel art resolution)
- Window override: 1280x960 (4x scale)
- Stretch mode: viewport with expand aspect
- Texture filter: nearest neighbor (pixel art)

## Godot Plugins

- **AsepriteWizard**: Imports Aseprite files for sprite animations
- **ldtk-importer**: Imports LDtk level data to Godot scenes

## Key Patterns

### GDExtension Reloadability (Development Workflow)

The `.gdextension` file sets `reloadable = true`, allowing Rust code changes to hot-reload during development (requires Godot 4.1+). This means you can modify Rust code, rebuild with `cargo build`, and see changes in the running Godot editor without restarting the engine. This is critical for iteration speed.

### Player Movement State Machine

The `PlayerMovement` struct implements a simple two-state machine:
- **Air**: Falling/jumping, allows limited air control
- **Floor**: On ground, full movement control and jump initiation

State transitions happen in `physics_process` based on `is_on_floor()`.

### Room Transitions and Grid-Based Navigation

The game uses a grid-based room system where each room is loaded dynamically:
- `RoomLoader` caches and loads scenes by grid coordinates: `"res://pipeline/ldtk/levels/Room_{x}_{y}.scn"`
- Adjacent rooms are calculated from current position/grid coordinates (no hardcoded connections)
- `BoundaryDetector` triggers room transitions when player crosses boundaries
- `Portal` entities enable explicit room-to-room teleportation; emit `teleport_requested` signal on interaction

The `Game` node orchestrates room management via `RoomManager` and provides utility functions for:
- `PlayerSpawner` - Initial player placement
- `SpawnResolver` - Resolving spawn points based on entry method (checkpoint load vs. portal entry)
- `PortalConnector` - Linking portal destinations across rooms

### Animation Integration

Player sprite animations are driven by velocity and movement state:
- Horizontal velocity controls sprite flipping via `set_scale`
- `get_animation_name` maps (velocity, state) → animation name
- Animations: "idle", "walk", "jump", "fall", "death"

### Player Death and Hazard System

Players die when touching hazard tiles (specific tilemaps identified by checking the tilemap node name). The death flow:
1. `check_hazard_collision()` detects collisions with hazard layers (identified by tilemap node checking)
2. `start_death()` plays the death animation and emits the `death_finished` signal
3. Game logic (outside Player class) listens to `death_finished` to reload checkpoints or restart levels
4. Player can be respawned by loading a saved checkpoint or repositioning via game state

### Save and Checkpoint System

The save system uses a slot-based architecture with thread-local in-memory storage:

**Save Flow (Checkpoints)**:
1. Player touches `Checkpoint` Area2D
2. `Checkpoint::activate()` calls `save::save_checkpoint()` with room coords and position
3. Save data stored in `SaveStore` (thread-local via `RefCell`)
4. Checkpoint sprite switches from "unchecked" to "checked" animation

**Load Flow**:
1. When checkpoint scene loads, `Checkpoint::ready()` calls `restore_if_saved()`
2. Compares saved room coords and position (within `POSITION_MATCH_EPSILON = 1.0`)
3. If match found, applies saved state (sets `activated = true`, plays "checked" animation)

**Entity State Persistence**:
- Keys/Locks use `EntityId = (room_x, room_y, pos_x, pos_y)` for identification
- `mark_key_collected()` / `is_key_collected()` - Track collected keys across room transitions
- `mark_lock_unlocked()` / `is_lock_unlocked()` - Track unlocked locks across room transitions
- `mark_star_collected()` / `is_star_collected()` - Track collected stars for objective/progression tracking
- Entities check saved state in `ready()` and `queue_free()` if already collected/unlocked

**Key Constants**:
- `DEFAULT_SAVE_SLOT = 0` - Currently only one save slot is used
- `POSITION_MATCH_EPSILON = 1.0` - Distance tolerance for matching checkpoints

**Godot Integration**:
- `SaveApi` class provides Godot-callable methods: `has_save()`, `queue_load()`, `clear_pending_load()`
- `reset_all()` clears all save state for new game

## Project Structure Notes

- Rust source uses edition 2024
- Optimization level 1 enabled for debug builds to improve development iteration speed
- Git ignores AI assistant files (`AGENTS.md`, `CLAUDE.md`, `GEMINI.md`)
