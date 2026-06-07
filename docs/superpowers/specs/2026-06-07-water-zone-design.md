# WaterZone Design

## Context

`p1proto` is a Godot 4 platformer whose runtime gameplay logic lives mostly in the Rust GDExtension. Room geometry is authored in LDtk and imported into generated Godot scenes under `godot/pipeline/ldtk/levels`.

`Room_1_2` is a 320x240 room. Its lower half is currently an open vertical space from about `y=120` down to a solid floor at `y=232`. The first water-system target is to place a configurable water body in that lower section and make it useful as traversal support, inspired by the way water can function as a traversal surface in Thomas Was Alone.

## Goals

- Add a reusable `WaterZone` entity that is configured in LDtk, not hard-coded to `Room_1_2`.
- Let level authors place and resize rectangular water bodies in LDtk.
- Make water useful instead of purely hazardous:
  - The water surface behaves like a soft platform when the player lands on it from above.
  - The player can jump from the water surface.
  - The player can move horizontally while inside the water body.
- Keep the first implementation small enough to validate feel before adding richer water mechanics.
- Keep deterministic water-contact rules testable in Rust without a live Godot runtime where practical.

## Non-Goals

- No rising, falling, or animated water level.
- No fluid simulation.
- No drowning timer.
- No per-character buoyancy differences.
- No current, flow, or wave physics.
- No broad redesign of room loading, hazards, ladders, or player movement.

## Authoring Workflow

LDtk gets a new entity definition named `WaterZone`.

- Default size: a practical rectangular starting size, such as `160x64`.
- Resizing: both horizontal and vertical resizing enabled.
- Rendering in LDtk: rectangle mode with a readable water-blue color.
- First placement: one `WaterZone` in the lower half of `Room_1_2`.

The existing `godot/pipeline/ldtk/entities_post_import.gd` import script will recognize `WaterZone`, instantiate `res://entity/water_zone.tscn`, and copy the LDtk entity rectangle size into exported scene properties.

## Runtime Architecture

### `godot/entity/water_zone.tscn`

The scene root is a Rust-backed `WaterZone` Godot class using `Area2D` as its base.

Children:

- `CollisionShape2D` for the rectangular water volume.
- A simple visible node for the first pass, such as a colored rectangle or sprite-backed fill, so the water body is easy to read while testing.

The scene should be a reusable entity shell, matching the existing pattern used by entities such as `Ladder`, `Checkpoint`, and `MovingPlatform`.

### `rust/src/entity/water_zone.rs`

`WaterZone` owns the water-volume data exposed from LDtk.

Responsibilities:

- Export `width_px` and `height_px`.
- Resize the `CollisionShape2D` to match the LDtk rectangle.
- Resize or configure the visible water fill to match the same rectangle.
- Add itself to the group `"water_zone"` during setup.
- Provide small query helpers such as `bounds()`, `surface_y()`, and player-overlap checks.

`rust/src/entity/mod.rs` will expose the new module.

### `rust/src/player/water.rs`

Water contact and movement rules should live in a small player submodule rather than being scattered through `Player::physics_process`.

Responsibilities:

- Find overlapping `WaterZone` nodes through the `"water_zone"` group.
- Resolve the current `WaterContact`:
  - `None`
  - `Surface`
  - `Submerged`
- Provide pure helper functions for surface snap, submerged detection, and movement tuning.

Suggested first-pass tuning values:

- `surface_snap_depth`: how far below the top edge the player can be and still be treated as standing on the water surface.
- `swim_horizontal_speed_multiplier`: the horizontal speed scale while submerged.
- `buoyancy_velocity`: the upward or gravity-countering velocity applied while submerged.

## Player Behavior

Water is checked after ladder state has had a chance to run, and before normal platformer movement fully resolves ground physics. Ladders remain more explicit than water because climbing is an intentional interaction, while water is an environmental traversal state.

### Surface

If the player is inside a `WaterZone` horizontally and near its top edge while falling or resting, the water surface acts as a soft platform:

- Clamp or snap the player to the water surface.
- Clear downward velocity.
- Treat the player as jump-capable for this frame.
- Allow a jump from the surface to return the player to ordinary platformer motion.

The water surface is not implemented as one-way collision. The player code owns the surface response so it can stay deterministic and avoid fighting Godot collision resolution.

### Submerged

When the player is below the water surface and inside the water rectangle:

- Horizontal movement remains available, with reduced speed or acceleration.
- Gravity is reduced or countered by a buoyancy velocity.
- Pressing upward or jump can help the player rise toward the surface.
- Pressing downward can allow the player to stay submerged or descend.
- The player cannot repeatedly jump while submerged; jump restoration happens at the surface state.

### Leaving Water

When the player exits all water zones:

- Clear transient water state.
- Restore normal platformer movement.
- Preserve existing behavior for hazards, aim indicator, ladder regrab blocking, room transitions, and quick respawn.

## Integration Points

- Add the `WaterZone` entity definition and first `Room_1_2` placement in `godot/pipeline/ldtk/tilemap.ldtk`.
- Update `godot/pipeline/ldtk/entities_post_import.gd` with a `setup_water_zone` path.
- Add `godot/entity/water_zone.tscn`.
- Add `rust/src/entity/water_zone.rs` and register it in `rust/src/entity/mod.rs`.
- Add `rust/src/player/water.rs` and wire it into `rust/src/player/mod.rs`.
- Regenerate or update the imported room scene so `godot/pipeline/ldtk/levels/Room_1_2.scn` contains the water entity.

## Testing And Verification

Rust unit tests should cover the deterministic rules:

- Detect surface contact only when the player is horizontally inside the water and within the snap depth near the top edge.
- Reject surface contact when the player is too deep, outside the horizontal range, or moving upward away from the surface.
- Detect submerged contact when the player is inside the rectangle below the surface band.
- Confirm submerged movement tuning does not restore jump repeatedly.
- Confirm leaving water clears water contact.

Manual verification should cover:

- `Room_1_2` loads with the visible water body in the lower half.
- Falling onto the water surface lets the player stand/float and jump.
- Moving inside the water allows horizontal adjustment.
- Leaving the water restores ordinary movement.
- Ladders, hazards, checkpoint respawn, and quick respawn still behave normally.

Required command verification:

- `cd rust; cargo fmt --check`
- `cd rust; cargo test --locked`

If Godot is available on `PATH`, also use the existing run workflow to verify the generated room can load:

- `./scripts/run.ps1`

