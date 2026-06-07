# Pixel Water Animation Design

## Context

`p1proto` already has a `WaterZone` direction: LDtk-authored rectangular water volumes, Rust-side player contact rules, and a first-pass Godot scene. That first pass is functional, but its visible layer is still a simple colored rectangle. This design upgrades the water into a pixel-art animated water body with player-driven splash feedback.

The visual target is closer to the readable water treatment in `Thomas Was Alone`: strong geometric readability, animated surface feedback, and clear player contact reactions. The chosen direction is `Pixel Surface` with `Readable Splash` intensity. The water should feel authored and reactive, not like a flat blue debug area.

## Goals

- Replace the placeholder water fill with a tile-friendly pixel water presentation.
- Keep water dimensions authored through LDtk and synchronized through `WaterZone`.
- Add clear but restrained splash feedback when the player enters, lands on, or exits water.
- Add low-frequency bubbles or short water lines while the player swims.
- Keep gameplay rules deterministic and testable in Rust.
- Keep visual assets aligned with the existing Aseprite pipeline and Godot entity scene style.

## Non-Goals

- No fluid simulation.
- No shader-heavy water for this pass.
- No physics waves that change collision or buoyancy.
- No drowning, currents, water level animation, or new player abilities.
- No broad rewrite of player movement, LDtk import, or room loading.

## Visual Direction

Water should use a chunky pixel style that fits the existing 8px room/tile scale.

The water body has three visible layers:

- Surface crest: a horizontally repeating animated wave strip with bright cyan highlights and darker trough pixels.
- Body fill: a deeper blue fill with sparse horizontal bands or tileable shimmer marks.
- Depth accents: occasional darker rows or bubbles so tall water bodies do not read as a single flat rectangle.

The animation should loop quietly. The surface can advance in 2 to 4 chunky frames. The body accents should move slowly or cycle through a short frame set. If the water zone is resized, these visuals repeat or stretch in controlled pixel increments rather than scaling the source art into blurry shapes.

## Splash Feedback

Splash intensity is `Readable Splash`: clear feedback without large arcade-style bursts.

Events:

- `EnterSurface`: when the player first contacts the water surface from above or lands into the surface snap band.
- `Dive`: when the player moves from surface contact into a submerged state.
- `ExitWater`: when the player leaves the water upward or horizontally.
- `SwimTick`: a throttled event while submerged and moving.

Visual response:

- `EnterSurface` plays the main splash: a short upward pixel droplet burst plus a ring ripple on the waterline.
- `Dive` plays a smaller downward disturbance with a few bubbles.
- `ExitWater` plays a smaller break-surface effect with a short ripple and two or three droplets.
- `SwimTick` plays subtle bubbles or short horizontal water lines at a limited cadence.

`SwimTick` must be throttled so the player cannot create continuous visual noise every physics frame. The first implementation should use a default cooldown of `0.25` seconds, so active underwater movement creates at most 4 small events per second.

## Architecture

The feature is split into gameplay state, water entity, and visual playback.

### `rust/src/player/water.rs`

This module continues to own deterministic contact resolution and movement tuning. It gains an event-oriented layer that compares previous and current water state and produces water visual events.

Core types:

- `WaterContact`: existing contact state: `None`, `Surface`, and `Submerged`.
- `WaterEventKind`: `EnterSurface`, `Dive`, `ExitWater`, `SwimTick`.
- `WaterEvent`: event kind plus global position and enough context for visual placement.
- `WaterEventState`: remembers previous contact and `SwimTick` cooldown.
- `ResolvedWaterContact`: contact plus the water zone that produced it, so event routing does not need a second broad water-zone lookup.

Pure tests should cover state transitions and throttling.

### `rust/src/player/mod.rs`

`Player::physics_process` should keep water contact resolution close to the existing movement integration. After resolving current contact, it asks `water.rs` for visual events and forwards them to the current `WaterZone`.

The player should not know about individual sprite nodes or animation names. It only emits water visual events at useful positions:

- Surface events use the water surface Y and the player X.
- Submerged movement events use the player body center or a slightly offset position.
- Exit events use the last known water surface if available.

### `rust/src/entity/water_zone.rs`

`WaterZone` remains the bridge between LDtk-authored water rectangles and Godot scene nodes. It should synchronize both collision and visuals from `width_px` and `height_px`.

New responsibilities:

- Replace the placeholder `ColorRect` sync with visual-node sync.
- Expose `play_water_event(kind, global_position)`.
- Convert global event positions to local coordinates and clamp them to the water bounds.
- Route the event to the correct visual player.

`WaterZone` should remain robust if optional visual nodes are missing: log a warning in development, skip playback, and keep gameplay functional.

### `godot/entity/water_zone.tscn`

The scene root remains `WaterZone` with `Area2D` base. The first implementation should use explicit sprite containers rather than shaders or scaled rectangles. Children should include:

- `CollisionShape2D`: rectangular water volume.
- `SurfaceTiles`: `Node2D` container whose children are repeated `Sprite2D` or `AnimatedSprite2D` surface tile instances.
- `FillTiles`: `Node2D` container whose children are repeated `Sprite2D` fill/accent tile instances.
- `SplashPlayer`: `AnimatedSprite2D` used for one-shot surface splash animations.
- `BubblePlayer`: `AnimatedSprite2D` used for one-shot submerged bubble or water-line animations.

`WaterZone` should rebuild or reposition `SurfaceTiles` and `FillTiles` when `width_px` or `height_px` changes. Tile instances must preserve nearest-neighbor pixel rendering and must not be scaled non-uniformly.

## Asset Pipeline

Use the existing Aseprite workflow style under `godot/pipeline/aseprite`.

Add this source-generation script:

- `godot/pipeline/aseprite/scripts/create_water.lua`

Generated or exported assets should produce reusable Godot resources under:

- `godot/pipeline/aseprite/src/water.aseprite`
- `godot/pipeline/aseprite/wizard/water.png`
- `godot/pipeline/aseprite/wizard/water.res`

The asset should include named animations or slices for:

- surface loop
- body fill or accent tiles
- enter splash
- exit splash
- dive bubbles
- swim bubbles

If a single `water.res` becomes awkward to address from Godot, split the visual resources into a small number of named resources, but keep the source-generation workflow centralized.

## Data Flow

1. LDtk defines and sizes `WaterZone`.
2. The LDtk post-import script instantiates `godot/entity/water_zone.tscn` and writes `width_px` and `height_px`.
3. `WaterZone` syncs collision and visible water layers to the authored size.
4. Each player physics frame resolves water contact against water zones.
5. Water contact transition logic emits zero or more visual events.
6. `Player` forwards events to the matching `WaterZone`.
7. `WaterZone` plays local splash or bubble animations at the clamped event position.

## Error Handling

- Missing visual nodes should not break gameplay water movement.
- Invalid or non-finite water sizes should continue to normalize to a minimum positive size.
- If multiple water zones overlap, events should target the same zone that produced the selected contact. Contact resolution should therefore return both the contact state and the selected water zone.
- If an event position falls outside the visual bounds, clamp it to the nearest point on the water rectangle so splashes remain attached to the water.

## Testing

Rust unit tests should cover:

- `None -> Surface` emits exactly one `EnterSurface`.
- `Surface -> Submerged` emits `Dive`.
- `Surface/Submerged -> None` emits `ExitWater`.
- Repeated `Submerged` frames do not emit `SwimTick` until the cooldown expires.
- `SwimTick` requires meaningful movement input or player velocity. The first implementation should treat horizontal input with absolute value at least `0.2`, vertical input with absolute value at least `0.2`, or velocity length at least `10 px/s` as meaningful movement.
- Event positions use the expected player/surface coordinates.

Godot verification should cover:

- `water_zone.tscn` contains the expected visual nodes and no longer relies on a bare `ColorRect` as the final visual.
- LDtk regeneration keeps water sizing intact.
- Room `Room_1_2` shows animated pixel water rather than a flat blue rectangle.
- Landing, diving, exiting, and swimming trigger the intended effects without hiding the player.

Required command verification:

- `cd rust; cargo fmt --check`
- `cd rust; cargo test --locked`
- Godot import/regeneration command if Godot is available on `PATH`.

Manual verification:

- Run the game through `./scripts/run.ps1` if Godot is available on `PATH` and the command can be stopped after manual inspection.
- In the water test room, fall onto water, jump out, dive, and swim horizontally.
- Confirm water feedback is visible but not noisy, and ordinary platforming readability remains intact.
