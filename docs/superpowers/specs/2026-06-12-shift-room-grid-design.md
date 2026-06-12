# Shift Room Grid Design

## Status

Approved for implementation planning.

## Goal

Add a lightweight gameplay view mode where holding Shift overlays grid lines across the current game room. The feature is a temporary readability aid for platforming and puzzle spacing. It should feel like a simple in-world alignment layer, not a map screen.

## Reference

The reference is Leap Year's Shift view only in the narrow sense of a visible grid over the playable space. This design does not attempt to recreate Leap Year's zoomed-out world view, color-region abstraction, icon layer, or bottom timeline UI.

## Player Experience

When the player holds Shift, thin grid lines appear over the current room. Releasing Shift hides them immediately. The game keeps running normally while the grid is visible.

The grid should be subtle enough that the player, terrain, water, doors, keys, stars, ladders, and other room content remain readable. It should be strong enough to make tile spacing and jump planning easy to judge.

## Scope

In scope:

- Add a dedicated input action for the grid view, bound to Shift.
- Draw grid lines over the current room while the action is held.
- Align the grid to the room's pixel coordinate system.
- Use an 8 px grid interval to match the LDtk tile size.
- Cover the current room's 320x240 playable area.

Out of scope:

- No world map behavior.
- No camera zoom, pan, or room preview.
- No adjacent-room display.
- No room selection.
- No icon layer for entities or collectibles.
- No pause or slow motion.
- No Leap Year-style colored region abstraction or hand-drawn outlines.

## Architecture

Implement the feature as a small UI/control overlay in the Godot scene tree, consistent with the existing Rust-backed UI nodes. The overlay should live under `game.tscn` alongside HUD-style nodes, not inside individual imported room scenes.

The overlay owns only presentation and input visibility. It should not know about room transitions, save state, entity state, or player movement. `GameRoomManager` remains the owner of active room lifecycle.

## Components

`RoomGridOverlay`:

- A Rust `GodotClass` based on `Control`.
- Hidden by default.
- Processes while gameplay is running.
- Reads the grid input action each frame.
- Queues redraw when visibility changes or the control size changes.
- Draws vertical and horizontal lines at the configured interval.

Scene wiring:

- Add a `RoomGridOverlay` instance to `godot/game.tscn`.
- Keep it visually above the room and player, but below blocking menus such as pause and world map.

Input:

- Add `ui_grid_view` to `godot/project.godot`.
- Bind it to Shift.
- Treat the action as hold-only, using pressed state rather than toggle state.

## Rendering Details

The grid interval defaults to `8.0` px. The default line color should be a low-alpha neutral light color so it reads over the existing level art without taking over the screen.

The grid should draw from the room origin, not from the window origin after scaling. Because the project viewport is 320x240 and rooms are currently 320x240, drawing in the full `Control` rect is sufficient for the first implementation. The exported room size and grid interval should still be configurable so the feature can follow future room-size changes.

## Data Flow

1. Godot input reports whether `ui_grid_view` is pressed.
2. `RoomGridOverlay` updates its own visibility.
3. When visible, `RoomGridOverlay` draws a fixed 8 px grid over its configured room rect.
4. Releasing Shift hides the overlay without touching game state.

## Error Handling

If the input action is missing, the overlay should remain hidden and avoid crashing. This is primarily a development-time configuration error.

If the control size differs from the configured room size, the overlay should still draw inside its available rect rather than failing. The first implementation can keep the default room size at 320x240.

## Testing

Add Rust unit coverage for any pure helper that computes grid line positions, especially:

- Includes both outer edges when room dimensions are exact multiples of the interval.
- Handles non-multiple dimensions by including the final room edge.
- Rejects or clamps invalid intervals so drawing cannot loop forever.

Manual verification:

- Run the game.
- Hold Shift in a room and confirm an 8 px grid fills the visible room.
- Release Shift and confirm the grid disappears immediately.
- Confirm gameplay continues while Shift is held.
- Open pause/menu map states and confirm the grid does not visually override blocking UI.
