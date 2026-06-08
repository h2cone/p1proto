use godot::prelude::*;

use super::MovementInput;

pub const PLAYER_HALF_HEIGHT_PX: f32 = 12.0;
pub const SWIM_TICK_COOLDOWN_SECONDS: f64 = 0.25;
const WATER_INPUT_THRESHOLD: f32 = 0.2;
const WATER_MOVEMENT_INPUT_THRESHOLD: f32 = 0.2;
const WATER_MOVEMENT_VELOCITY_THRESHOLD: f32 = 10.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaterBounds {
    pub center: Vector2,
    pub size: Vector2,
}

impl WaterBounds {
    pub fn from_center_size(center: Vector2, size: Vector2) -> Self {
        Self { center, size }
    }

    pub fn left(&self) -> f32 {
        self.center.x - self.size.x * 0.5
    }

    pub fn right(&self) -> f32 {
        self.center.x + self.size.x * 0.5
    }

    pub fn top(&self) -> f32 {
        self.center.y - self.size.y * 0.5
    }

    pub fn bottom(&self) -> f32 {
        self.center.y + self.size.y * 0.5
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WaterContact {
    None,
    Surface { surface_y: f32 },
    Submerged,
}

impl WaterContact {
    pub fn is_surface(self) -> bool {
        matches!(self, Self::Surface { .. })
    }

    pub fn is_submerged(self) -> bool {
        matches!(self, Self::Submerged)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedWaterContact {
    pub contact: WaterContact,
    pub zone_index: Option<usize>,
}

impl ResolvedWaterContact {
    pub fn none() -> Self {
        Self {
            contact: WaterContact::None,
            zone_index: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaterEventKind {
    EnterSurface,
    Dive,
    ExitWater,
    SwimTick,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaterEvent {
    pub kind: WaterEventKind,
    pub position: Vector2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaterOverlayMask {
    pub local_top_y: f32,
    pub covered_height: f32,
    pub show_surface_line: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct WaterTuning {
    pub surface_snap_depth: f32,
    pub surface_float_depth: f32,
    pub surface_horizontal_speed_multiplier: f32,
    pub submerged_horizontal_speed_multiplier: f32,
    pub buoyancy_velocity: f32,
    pub swim_rise_velocity: f32,
    pub swim_descend_velocity: f32,
}

impl Default for WaterTuning {
    fn default() -> Self {
        Self {
            surface_snap_depth: 12.0,
            surface_float_depth: 8.0,
            surface_horizontal_speed_multiplier: 0.9,
            submerged_horizontal_speed_multiplier: 0.65,
            buoyancy_velocity: -40.0,
            swim_rise_velocity: -90.0,
            swim_descend_velocity: 70.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WaterState {
    contact: WaterContact,
    last_surface_y: Option<f32>,
    swim_tick_cooldown: f64,
}

impl Default for WaterState {
    fn default() -> Self {
        Self {
            contact: WaterContact::None,
            last_surface_y: None,
            swim_tick_cooldown: 0.0,
        }
    }
}

impl WaterState {
    pub fn update(&mut self, contact: WaterContact) {
        self.contact = contact;
        if let WaterContact::Surface { surface_y } = contact {
            self.last_surface_y = Some(surface_y);
        }
        if contact == WaterContact::None {
            self.last_surface_y = None;
            self.swim_tick_cooldown = 0.0;
        }
    }

    pub fn update_and_events(
        &mut self,
        contact: WaterContact,
        player_position: Vector2,
        velocity: Vector2,
        input: MovementInput,
        delta: f64,
    ) -> Vec<WaterEvent> {
        let previous = self.contact;
        let previous_surface_y = self.last_surface_y;
        self.swim_tick_cooldown = (self.swim_tick_cooldown - delta).max(0.0);

        let mut events = Vec::new();
        match (previous, contact) {
            (WaterContact::None, WaterContact::Surface { surface_y }) => {
                events.push(WaterEvent {
                    kind: WaterEventKind::EnterSurface,
                    position: Vector2::new(player_position.x, surface_y),
                });
            }
            (WaterContact::Surface { .. }, WaterContact::Submerged) => {
                events.push(WaterEvent {
                    kind: WaterEventKind::Dive,
                    position: player_position,
                });
            }
            (WaterContact::Surface { .. } | WaterContact::Submerged, WaterContact::None) => {
                events.push(WaterEvent {
                    kind: WaterEventKind::ExitWater,
                    position: Vector2::new(
                        player_position.x,
                        previous_surface_y.unwrap_or(player_position.y),
                    ),
                });
            }
            _ => {}
        }

        if contact == WaterContact::Submerged
            && previous == WaterContact::Submerged
            && self.swim_tick_cooldown <= 0.0
            && has_meaningful_swim_movement(input, velocity)
        {
            events.push(WaterEvent {
                kind: WaterEventKind::SwimTick,
                position: player_position,
            });
            self.swim_tick_cooldown = SWIM_TICK_COOLDOWN_SECONDS;
        }

        self.update(contact);
        events
    }
}

impl Default for WaterContact {
    fn default() -> Self {
        Self::None
    }
}

#[allow(dead_code)]
pub fn resolve_contact(
    player_position: Vector2,
    velocity: Vector2,
    bounds: impl IntoIterator<Item = WaterBounds>,
    snap_depth: f32,
) -> WaterContact {
    let mut submerged = false;

    for water_bounds in bounds {
        match contact_for_bounds(player_position, velocity, water_bounds, snap_depth) {
            WaterContact::Surface { surface_y } => return WaterContact::Surface { surface_y },
            WaterContact::Submerged => submerged = true,
            WaterContact::None => {}
        }
    }

    if submerged {
        WaterContact::Submerged
    } else {
        WaterContact::None
    }
}

pub fn resolve_targeted_contact(
    player_position: Vector2,
    velocity: Vector2,
    bounds: impl IntoIterator<Item = (usize, WaterBounds)>,
    snap_depth: f32,
) -> ResolvedWaterContact {
    let mut submerged_zone_index = None;

    for (zone_index, water_bounds) in bounds {
        match contact_for_bounds(player_position, velocity, water_bounds, snap_depth) {
            WaterContact::Surface { surface_y } => {
                return ResolvedWaterContact {
                    contact: WaterContact::Surface { surface_y },
                    zone_index: Some(zone_index),
                };
            }
            WaterContact::Submerged => {
                if submerged_zone_index.is_none() {
                    submerged_zone_index = Some(zone_index);
                }
            }
            WaterContact::None => {}
        }
    }

    if let Some(zone_index) = submerged_zone_index {
        ResolvedWaterContact {
            contact: WaterContact::Submerged,
            zone_index: Some(zone_index),
        }
    } else {
        ResolvedWaterContact::none()
    }
}

pub fn contact_for_bounds(
    player_position: Vector2,
    velocity: Vector2,
    bounds: WaterBounds,
    snap_depth: f32,
) -> WaterContact {
    if !is_horizontally_inside(player_position.x, bounds) {
        return WaterContact::None;
    }

    let feet_y = player_position.y + PLAYER_HALF_HEIGHT_PX;
    let top_y = bounds.top();

    if velocity.y >= 0.0 && feet_y >= top_y - snap_depth && feet_y <= top_y + snap_depth {
        return WaterContact::Surface { surface_y: top_y };
    }

    let player_top = player_position.y - PLAYER_HALF_HEIGHT_PX;
    let player_bottom = player_position.y + PLAYER_HALF_HEIGHT_PX;
    let overlaps_vertically = player_top <= bounds.bottom() && player_bottom >= bounds.top();
    if overlaps_vertically && feet_y > top_y + snap_depth {
        WaterContact::Submerged
    } else {
        WaterContact::None
    }
}

pub fn velocity_for_surface(mut velocity: Vector2, contact: WaterContact) -> Vector2 {
    if contact.is_surface() && velocity.y > 0.0 {
        velocity.y = 0.0;
    }
    velocity
}

pub fn surface_float_center_y(surface_y: f32, tuning: WaterTuning) -> f32 {
    surface_y + tuning.surface_float_depth - PLAYER_HALF_HEIGHT_PX
}

pub fn overlay_mask_for_contact(
    contact: WaterContact,
    player_center_y: f32,
) -> Option<WaterOverlayMask> {
    match contact {
        WaterContact::None => None,
        WaterContact::Surface { surface_y } => {
            let local_top_y =
                (surface_y - player_center_y).clamp(-PLAYER_HALF_HEIGHT_PX, PLAYER_HALF_HEIGHT_PX);
            Some(WaterOverlayMask {
                local_top_y,
                covered_height: PLAYER_HALF_HEIGHT_PX - local_top_y,
                show_surface_line: true,
            })
        }
        WaterContact::Submerged => Some(WaterOverlayMask {
            local_top_y: -PLAYER_HALF_HEIGHT_PX,
            covered_height: PLAYER_HALF_HEIGHT_PX * 2.0,
            show_surface_line: false,
        }),
    }
}

pub fn velocity_for_surface_float(
    mut velocity: Vector2,
    input: MovementInput,
    tuning: WaterTuning,
    jump_velocity: f32,
) -> Vector2 {
    velocity.x *= tuning.surface_horizontal_speed_multiplier;

    if input.jump_just_pressed {
        velocity.y = jump_velocity;
    } else if is_dive_input(input) {
        velocity.y = tuning.swim_descend_velocity;
    } else {
        velocity.y = 0.0;
    }

    velocity
}

pub fn should_snap_to_surface_float(input: MovementInput) -> bool {
    !is_dive_input(input)
}

pub fn velocity_for_submerged(
    mut velocity: Vector2,
    input: MovementInput,
    tuning: WaterTuning,
) -> Vector2 {
    velocity.x *= tuning.submerged_horizontal_speed_multiplier;

    if input.vertical_direction <= -WATER_INPUT_THRESHOLD || input.jump_just_pressed {
        velocity.y = velocity.y.min(tuning.swim_rise_velocity);
    } else if input.vertical_direction >= WATER_INPUT_THRESHOLD {
        velocity.y = velocity.y.min(tuning.swim_descend_velocity);
    } else if velocity.y > tuning.buoyancy_velocity {
        velocity.y = tuning.buoyancy_velocity;
    }

    velocity
}

pub fn input_without_regular_jump(mut input: MovementInput) -> MovementInput {
    input.jump_just_pressed = false;
    input.jump_just_released = false;
    input
}

fn is_horizontally_inside(player_x: f32, bounds: WaterBounds) -> bool {
    player_x >= bounds.left() && player_x <= bounds.right()
}

fn is_dive_input(input: MovementInput) -> bool {
    input.vertical_direction >= WATER_INPUT_THRESHOLD
}

fn has_meaningful_swim_movement(input: MovementInput, velocity: Vector2) -> bool {
    input.direction.abs() >= WATER_MOVEMENT_INPUT_THRESHOLD
        || input.vertical_direction.abs() >= WATER_MOVEMENT_INPUT_THRESHOLD
        || velocity.length() >= WATER_MOVEMENT_VELOCITY_THRESHOLD
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::MovementInput;

    fn test_bounds() -> WaterBounds {
        WaterBounds::from_center_size(Vector2::new(100.0, 160.0), Vector2::new(160.0, 64.0))
    }

    #[test]
    fn detects_surface_contact_when_feet_are_inside_snap_band_and_falling() {
        let contact = contact_for_bounds(
            Vector2::new(100.0, 120.0),
            Vector2::new(0.0, 20.0),
            test_bounds(),
            WaterTuning::default().surface_snap_depth,
        );

        assert_eq!(contact, WaterContact::Surface { surface_y: 128.0 });
    }

    #[test]
    fn rejects_surface_contact_when_too_deep_outside_or_rising() {
        let snap_depth = WaterTuning::default().surface_snap_depth;

        assert_eq!(
            contact_for_bounds(
                Vector2::new(100.0, 140.1),
                Vector2::new(0.0, 20.0),
                test_bounds(),
                snap_depth,
            ),
            WaterContact::Submerged
        );
        assert_eq!(
            contact_for_bounds(
                Vector2::new(181.0, 120.0),
                Vector2::new(0.0, 20.0),
                test_bounds(),
                snap_depth,
            ),
            WaterContact::None
        );
        assert_eq!(
            contact_for_bounds(
                Vector2::new(100.0, 120.0),
                Vector2::new(0.0, -20.0),
                test_bounds(),
                snap_depth,
            ),
            WaterContact::None
        );
    }

    #[test]
    fn detects_submerged_contact_below_surface_band() {
        let contact = contact_for_bounds(
            Vector2::new(100.0, 141.0),
            Vector2::new(0.0, 20.0),
            test_bounds(),
            WaterTuning::default().surface_snap_depth,
        );

        assert_eq!(contact, WaterContact::Submerged);
    }

    #[test]
    fn submerged_velocity_tuning_does_not_restore_regular_jump() {
        let velocity = velocity_for_submerged(
            Vector2::new(120.0, 160.0),
            MovementInput {
                direction: 1.0,
                jump_just_pressed: true,
                ..Default::default()
            },
            WaterTuning::default(),
        );

        assert!(velocity.x < 120.0);
        assert!(velocity.y > -300.0);
    }

    #[test]
    fn surface_float_position_submerges_feet_below_waterline() {
        let tuning = WaterTuning::default();

        let center_y = surface_float_center_y(128.0, tuning);

        assert_eq!(
            center_y + PLAYER_HALF_HEIGHT_PX,
            128.0 + tuning.surface_float_depth
        );
    }

    #[test]
    fn surface_float_velocity_uses_water_tuning_instead_of_floor_motion() {
        let velocity = velocity_for_surface_float(
            Vector2::new(120.0, 160.0),
            MovementInput::default(),
            WaterTuning::default(),
            -300.0,
        );

        assert!(velocity.x < 120.0);
        assert_eq!(velocity.y, 0.0);
    }

    #[test]
    fn overlay_mask_for_surface_covers_body_below_waterline() {
        let overlay = overlay_mask_for_contact(WaterContact::Surface { surface_y: 128.0 }, 124.0)
            .expect("surface contact should show water overlay");

        assert_eq!(overlay.local_top_y, 4.0);
        assert_eq!(overlay.covered_height, 8.0);
        assert!(overlay.show_surface_line);
    }

    #[test]
    fn overlay_mask_for_submerged_covers_full_body_without_surface_line() {
        let overlay = overlay_mask_for_contact(WaterContact::Submerged, 150.0)
            .expect("submerged contact should show water overlay");

        assert_eq!(overlay.local_top_y, -PLAYER_HALF_HEIGHT_PX);
        assert_eq!(overlay.covered_height, PLAYER_HALF_HEIGHT_PX * 2.0);
        assert!(!overlay.show_surface_line);
    }

    #[test]
    fn overlay_mask_is_hidden_when_not_in_water() {
        assert_eq!(overlay_mask_for_contact(WaterContact::None, 124.0), None);
    }

    #[test]
    fn surface_and_submerged_horizontal_speed_can_be_tuned_independently() {
        let tuning = WaterTuning {
            surface_snap_depth: 12.0,
            surface_float_depth: 8.0,
            surface_horizontal_speed_multiplier: 0.9,
            submerged_horizontal_speed_multiplier: 0.5,
            buoyancy_velocity: -40.0,
            swim_rise_velocity: -90.0,
            swim_descend_velocity: 70.0,
        };

        let surface_velocity = velocity_for_surface_float(
            Vector2::new(120.0, 0.0),
            MovementInput::default(),
            tuning,
            -300.0,
        );
        let submerged_velocity =
            velocity_for_submerged(Vector2::new(120.0, 0.0), MovementInput::default(), tuning);

        assert_eq!(surface_velocity.x, 108.0);
        assert_eq!(submerged_velocity.x, 60.0);
    }

    #[test]
    fn surface_jump_uses_regular_jump_velocity_once() {
        let velocity = velocity_for_surface_float(
            Vector2::new(0.0, 0.0),
            MovementInput {
                jump_just_pressed: true,
                ..Default::default()
            },
            WaterTuning::default(),
            -300.0,
        );

        assert_eq!(velocity.y, -300.0);
    }

    #[test]
    fn leaving_water_clears_contact_state() {
        let mut state = WaterState::default();

        state.update(WaterContact::Submerged);
        assert_eq!(state.contact, WaterContact::Submerged);

        state.update(WaterContact::None);
        assert_eq!(state.contact, WaterContact::None);
    }

    #[test]
    fn targeted_contact_prefers_surface_zone_over_submerged_zone() {
        let deep =
            WaterBounds::from_center_size(Vector2::new(100.0, 130.0), Vector2::new(160.0, 64.0));
        let surface =
            WaterBounds::from_center_size(Vector2::new(100.0, 132.0), Vector2::new(160.0, 32.0));

        let resolved = resolve_targeted_contact(
            Vector2::new(100.0, 104.0),
            Vector2::new(0.0, 20.0),
            [(0, deep), (1, surface)],
            WaterTuning::default().surface_snap_depth,
        );

        assert_eq!(resolved.contact, WaterContact::Surface { surface_y: 116.0 });
        assert_eq!(resolved.zone_index, Some(1));
    }

    #[test]
    fn surface_entry_emits_one_enter_surface_event() {
        let mut state = WaterState::default();
        let events = state.update_and_events(
            WaterContact::Surface { surface_y: 128.0 },
            Vector2::new(100.0, 120.0),
            Vector2::new(0.0, 80.0),
            MovementInput::default(),
            1.0 / 60.0,
        );

        assert_eq!(
            events,
            vec![WaterEvent {
                kind: WaterEventKind::EnterSurface,
                position: Vector2::new(100.0, 128.0),
            }]
        );

        let repeated = state.update_and_events(
            WaterContact::Surface { surface_y: 128.0 },
            Vector2::new(100.0, 120.0),
            Vector2::ZERO,
            MovementInput::default(),
            1.0 / 60.0,
        );
        assert!(repeated.is_empty());
    }

    #[test]
    fn surface_to_submerged_emits_dive_event() {
        let mut state = WaterState::default();
        state.update_and_events(
            WaterContact::Surface { surface_y: 128.0 },
            Vector2::new(100.0, 120.0),
            Vector2::ZERO,
            MovementInput::default(),
            1.0 / 60.0,
        );

        let events = state.update_and_events(
            WaterContact::Submerged,
            Vector2::new(100.0, 144.0),
            Vector2::new(0.0, 40.0),
            MovementInput {
                vertical_direction: 1.0,
                ..Default::default()
            },
            1.0 / 60.0,
        );

        assert_eq!(
            events,
            vec![WaterEvent {
                kind: WaterEventKind::Dive,
                position: Vector2::new(100.0, 144.0),
            }]
        );
    }

    #[test]
    fn leaving_water_emits_exit_at_last_surface() {
        let mut state = WaterState::default();
        state.update_and_events(
            WaterContact::Surface { surface_y: 128.0 },
            Vector2::new(100.0, 120.0),
            Vector2::ZERO,
            MovementInput::default(),
            1.0 / 60.0,
        );

        let events = state.update_and_events(
            WaterContact::None,
            Vector2::new(104.0, 112.0),
            Vector2::new(0.0, -120.0),
            MovementInput::default(),
            1.0 / 60.0,
        );

        assert_eq!(
            events,
            vec![WaterEvent {
                kind: WaterEventKind::ExitWater,
                position: Vector2::new(104.0, 128.0),
            }]
        );
    }

    #[test]
    fn swim_tick_is_throttled_and_requires_movement() {
        let mut state = WaterState::default();
        state.update_and_events(
            WaterContact::Submerged,
            Vector2::new(100.0, 144.0),
            Vector2::ZERO,
            MovementInput::default(),
            1.0 / 60.0,
        );

        let still = state.update_and_events(
            WaterContact::Submerged,
            Vector2::new(100.0, 144.0),
            Vector2::ZERO,
            MovementInput::default(),
            0.25,
        );
        assert!(still.is_empty());

        let moving = state.update_and_events(
            WaterContact::Submerged,
            Vector2::new(100.0, 144.0),
            Vector2::new(12.0, 0.0),
            MovementInput::default(),
            0.25,
        );
        assert_eq!(
            moving,
            vec![WaterEvent {
                kind: WaterEventKind::SwimTick,
                position: Vector2::new(100.0, 144.0),
            }]
        );

        let throttled = state.update_and_events(
            WaterContact::Submerged,
            Vector2::new(100.0, 144.0),
            Vector2::new(12.0, 0.0),
            MovementInput::default(),
            0.01,
        );
        assert!(throttled.is_empty());
    }
}
