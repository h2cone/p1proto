mod aim_indicator;
mod animation;
mod corner_correction;
mod hazard;
mod input_adapter;
mod ladder;
mod platform;
mod push;
pub(crate) mod water;

pub use crate::core::player::{MovementConfig, MovementInput, MovementState, PlayerMovement};
pub use animation::AnimationNames;
pub use input_adapter::InputActions;

use godot::{
    classes::{
        AnimatedSprite2D, CharacterBody2D, ICharacterBody2D, Node2D, Polygon2D, ProjectSettings,
    },
    prelude::*,
};

use crate::entity::water_zone::{WATER_ZONE_GROUP, WaterZone};

use self::aim_indicator::{AimDirection, AimIndicator, AimInput};
use self::platform::PlatformDropController;

const MOVING_PLATFORM_LAYER: i32 = 4;
const HAZARD_LAYER: i32 = 12;
const DROP_THROUGH_DURATION: f64 = 0.35;
const PUSH_SPEED: f32 = 80.0;
const DEATH_ANIMATION: &str = "death";
const HAZARD_TILEMAP_PREFIXES: [&str; 2] = ["HazardsTiles", "Hazards"];
const WATER_BODY_OVERLAY_PATH: &str = "WaterBodyOverlay";
const WATER_SURFACE_OVERLAY_PATH: &str = "WaterSurfaceOverlay";
const PLAYER_HALF_WIDTH_PX: f32 = 8.0;
const WATER_SURFACE_OVERLAY_HEIGHT_PX: f32 = 1.0;

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Player {
    base: Base<CharacterBody2D>,
    movement: Option<PlayerMovement>,
    sprite: OnReady<Gd<AnimatedSprite2D>>,
    aim_indicator: Option<Gd<AimIndicator>>,
    water_body_overlay: Option<Gd<Polygon2D>>,
    water_surface_overlay: Option<Gd<Polygon2D>>,
    #[export]
    aim_indicator_distance: f32,
    #[export]
    water_surface_snap_depth: f32,
    #[export]
    water_surface_float_depth: f32,
    #[export]
    water_surface_horizontal_speed_multiplier: f32,
    #[export]
    water_submerged_horizontal_speed_multiplier: f32,
    #[export]
    water_buoyancy_velocity: f32,
    #[export]
    water_swim_rise_velocity: f32,
    #[export]
    water_swim_descend_velocity: f32,
    input_actions: InputActions,
    animation_names: AnimationNames,
    drop_controller: PlatformDropController,
    aim_direction: AimDirection,
    is_dying: bool,
    is_climbing: bool,
    ladder_regrab_blocked: bool,
    water_state: water::WaterState,
    last_water_zone: Option<Gd<WaterZone>>,
}

struct PlayerWaterContact {
    contact: water::WaterContact,
    zone: Option<Gd<WaterZone>>,
}

fn water_tuning_from_exports(
    surface_snap_depth: f32,
    surface_float_depth: f32,
    surface_horizontal_speed_multiplier: f32,
    submerged_horizontal_speed_multiplier: f32,
    buoyancy_velocity: f32,
    swim_rise_velocity: f32,
    swim_descend_velocity: f32,
) -> water::WaterTuning {
    let defaults = water::WaterTuning::default();

    water::WaterTuning {
        surface_snap_depth: finite_or_default(surface_snap_depth, defaults.surface_snap_depth)
            .max(0.0),
        surface_float_depth: finite_or_default(surface_float_depth, defaults.surface_float_depth)
            .max(0.0),
        surface_horizontal_speed_multiplier: finite_or_default(
            surface_horizontal_speed_multiplier,
            defaults.surface_horizontal_speed_multiplier,
        )
        .max(0.0),
        submerged_horizontal_speed_multiplier: finite_or_default(
            submerged_horizontal_speed_multiplier,
            defaults.submerged_horizontal_speed_multiplier,
        )
        .max(0.0),
        buoyancy_velocity: finite_or_default(buoyancy_velocity, defaults.buoyancy_velocity),
        swim_rise_velocity: finite_or_default(swim_rise_velocity, defaults.swim_rise_velocity),
        swim_descend_velocity: finite_or_default(
            swim_descend_velocity,
            defaults.swim_descend_velocity,
        ),
    }
}

fn finite_or_default(value: f32, default: f32) -> f32 {
    if value.is_finite() { value } else { default }
}

#[godot_api]
impl ICharacterBody2D for Player {
    fn init(base: Base<CharacterBody2D>) -> Self {
        let water_tuning = water::WaterTuning::default();

        Self {
            base,
            movement: None,
            sprite: OnReady::from_node("AnimatedSprite2D"),
            aim_indicator: None,
            water_body_overlay: None,
            water_surface_overlay: None,
            aim_indicator_distance: 12.0,
            water_surface_snap_depth: water_tuning.surface_snap_depth,
            water_surface_float_depth: water_tuning.surface_float_depth,
            water_surface_horizontal_speed_multiplier: water_tuning
                .surface_horizontal_speed_multiplier,
            water_submerged_horizontal_speed_multiplier: water_tuning
                .submerged_horizontal_speed_multiplier,
            water_buoyancy_velocity: water_tuning.buoyancy_velocity,
            water_swim_rise_velocity: water_tuning.swim_rise_velocity,
            water_swim_descend_velocity: water_tuning.swim_descend_velocity,
            input_actions: InputActions::default(),
            animation_names: AnimationNames::default(),
            drop_controller: PlatformDropController::new(
                DROP_THROUGH_DURATION,
                MOVING_PLATFORM_LAYER,
            ),
            aim_direction: AimDirection::default(),
            is_dying: false,
            is_climbing: false,
            ladder_regrab_blocked: false,
            water_state: water::WaterState::default(),
            last_water_zone: None,
        }
    }

    fn ready(&mut self) {
        self.movement = Some(PlayerMovement::new(MovementConfig::platformer(
            project_gravity(),
        )));

        let moving_platform_mask_default =
            self.base().get_collision_mask_value(MOVING_PLATFORM_LAYER);
        self.drop_controller
            .configure_mask_default(moving_platform_mask_default);

        let player = self.to_gd();
        self.sprite
            .signals()
            .animation_finished()
            .connect_other(&player, Self::on_animation_finished);
        if let Some(mut frames) = self.sprite.get_sprite_frames() {
            frames.set_animation_loop(DEATH_ANIMATION, false);
        }

        self.aim_indicator = self.base().try_get_node_as::<AimIndicator>("AimIndicator");
        if self.aim_indicator.is_none() {
            godot_warn!(
                "[Player] AimIndicator node not found - add a child AimIndicator node in the editor to enable the template"
            );
        }
        self.update_aim_indicator(AimInput::default());

        self.water_body_overlay = self
            .base()
            .try_get_node_as::<Polygon2D>(WATER_BODY_OVERLAY_PATH);
        self.water_surface_overlay = self
            .base()
            .try_get_node_as::<Polygon2D>(WATER_SURFACE_OVERLAY_PATH);
        if self.water_body_overlay.is_none() || self.water_surface_overlay.is_none() {
            godot_warn!(
                "[Player] water overlay nodes not found - add WaterBodyOverlay and WaterSurfaceOverlay to show submerged body occlusion"
            );
        }
        self.hide_water_overlay();

        godot_print!("[Player] ready")
    }

    fn physics_process(&mut self, delta: f64) {
        if self.is_dying {
            self.base_mut().set_velocity(Vector2::ZERO);
            self.set_aim_indicator_visible(false);
            self.hide_water_overlay();
            return;
        }

        if input_adapter::is_respawn_pressed(&self.input_actions) {
            self.start_death();
            return;
        }

        let movement_input = input_adapter::collect_movement_input(&self.input_actions);
        let mut body = self.to_gd().upcast::<CharacterBody2D>();
        let touching_ladder = self.is_touching_ladder();
        self.update_ladder_regrab_block(movement_input, touching_ladder);
        let mut jumped_from_ladder = false;

        if self.is_climbing {
            if touching_ladder {
                if movement_input.jump_just_pressed {
                    self.stop_climbing();
                    self.ladder_regrab_blocked = true;
                    jumped_from_ladder = true;
                } else {
                    self.physics_process_climb(movement_input);
                    return;
                }
            } else {
                self.stop_climbing();
            }
        }

        if ladder::should_start_climbing(
            movement_input,
            touching_ladder,
            self.ladder_regrab_blocked,
            jumped_from_ladder,
        ) {
            self.start_climbing(&mut body);
            self.physics_process_climb(movement_input);
            return;
        }

        let velocity = self.base().get_velocity();
        let mut is_on_floor = self.base().is_on_floor();

        self.drop_controller.update(
            &mut body,
            is_on_floor,
            delta,
            input_adapter::is_drop_through_pressed(&self.input_actions),
        );
        if self.drop_controller.is_active() {
            is_on_floor = false;
        }

        let water_tuning = self.water_tuning();
        let resolved_water = self.resolve_water_contact(velocity, water_tuning.surface_snap_depth);
        let water_contact = resolved_water.contact;
        let player_position_for_water_event = body.get_global_position();
        let water_events = self.water_state.update_and_events(
            water_contact,
            player_position_for_water_event,
            velocity,
            movement_input,
            delta,
        );

        let mut event_target = resolved_water
            .zone
            .clone()
            .or_else(|| self.last_water_zone.clone());
        if let Some(zone) = resolved_water.zone {
            self.last_water_zone = Some(zone);
        }

        if let Some(zone) = event_target.as_mut() {
            for event in water_events {
                zone.bind_mut().play_water_event(event.kind, event.position);
            }
        }

        if water_contact == water::WaterContact::None {
            self.last_water_zone = None;
        }

        if let water::WaterContact::Surface { surface_y } = water_contact
            && water::should_snap_to_surface_float(movement_input)
        {
            self.snap_to_water_surface_float(&mut body, surface_y, water_tuning);
        }

        let movement_velocity = water::velocity_for_surface(velocity, water_contact);
        let movement_input_for_physics =
            if water_contact.is_surface() || water_contact.is_submerged() {
                water::input_without_regular_jump(movement_input)
            } else {
                movement_input
            };

        let aim_input = input_adapter::collect_aim_input(&self.input_actions);
        let Some(movement) = self.movement.as_mut() else {
            return;
        };
        let mut new_velocity = movement.physics_process(
            movement_velocity,
            is_on_floor || jumped_from_ladder,
            delta,
            movement_input_for_physics,
        );
        if water_contact.is_surface() {
            new_velocity = water::velocity_for_surface_float(
                new_velocity,
                movement_input,
                water_tuning,
                movement.config.jump_velocity,
            );
        } else if water_contact.is_submerged() {
            new_velocity =
                water::velocity_for_submerged(new_velocity, movement_input, water_tuning);
        }

        self.base_mut().set_velocity(new_velocity);
        self.base_mut().move_and_slide();
        if water_contact == water::WaterContact::None {
            corner_correction::apply_after_slide(&mut body, new_velocity, movement_input.direction);
        }
        self.update_water_overlay(water_contact, body.get_global_position());

        let resolved_velocity = self.base().get_velocity();
        let is_on_floor_after_move = self.base().is_on_floor();
        let (state, is_walking) = {
            let Some(movement) = self.movement.as_mut() else {
                return;
            };
            movement.post_physics_update(is_on_floor_after_move);
            (
                movement.state,
                movement.is_walking_or_pressing(resolved_velocity, movement_input.direction),
            )
        };

        if self.check_hazard_collision() {
            self.start_death();
            return;
        }

        push::push_rigid_bodies(
            &mut body,
            input_adapter::get_push_direction(&self.input_actions),
            PUSH_SPEED,
        );
        self.update_aim_indicator(aim_input);

        let visual_direction_x =
            animation::resolve_visual_direction_x(movement_input.direction, resolved_velocity.x);
        animation::update_sprite_direction(&mut self.sprite, visual_direction_x);
        let anim = animation::get_animation_name(
            state,
            resolved_velocity,
            is_walking,
            &self.animation_names,
        );
        animation::play_animation_if_changed(&mut self.sprite, anim);
    }
}

fn project_gravity() -> f32 {
    let settings = ProjectSettings::singleton();
    settings.get("physics/2d/default_gravity").to::<f64>() as f32
}

fn set_polygon_rect(polygon: &mut Gd<Polygon2D>, left: f32, right: f32, top: f32, bottom: f32) {
    polygon.set_polygon(&PackedVector2Array::from_iter([
        Vector2::new(left, top),
        Vector2::new(right, top),
        Vector2::new(right, bottom),
        Vector2::new(left, bottom),
    ]));
}

#[godot_api]
impl Player {
    #[signal]
    pub(crate) fn death_finished();

    #[func]
    fn on_animation_finished(&mut self) {
        if !self.is_dying {
            return;
        }

        if self.sprite.get_animation() == DEATH_ANIMATION {
            self.is_dying = false;
            self.signals().death_finished().emit();
        }
    }

    pub(crate) fn reset_for_room_transition(&mut self) {
        if let Some(movement) = &mut self.movement {
            movement.reset_transient_state();
        }

        self.is_climbing = false;
        self.ladder_regrab_blocked = false;
        self.water_state.update(water::WaterContact::None);
        self.last_water_zone = None;
        self.hide_water_overlay();

        let mut body = self.to_gd().upcast::<CharacterBody2D>();
        self.drop_controller.reset(&mut body);
        self.set_aim_indicator_visible(false);
    }

    fn water_tuning(&self) -> water::WaterTuning {
        water_tuning_from_exports(
            self.water_surface_snap_depth,
            self.water_surface_float_depth,
            self.water_surface_horizontal_speed_multiplier,
            self.water_submerged_horizontal_speed_multiplier,
            self.water_buoyancy_velocity,
            self.water_swim_rise_velocity,
            self.water_swim_descend_velocity,
        )
    }

    fn update_water_overlay(&mut self, contact: water::WaterContact, player_position: Vector2) {
        let Some(mask) = water::overlay_mask_for_contact(contact, player_position.y) else {
            self.hide_water_overlay();
            return;
        };

        let top = mask.local_top_y;
        let bottom = top + mask.covered_height;
        if let Some(mut overlay) = self.water_body_overlay.clone() {
            set_polygon_rect(
                &mut overlay,
                -PLAYER_HALF_WIDTH_PX,
                PLAYER_HALF_WIDTH_PX,
                top,
                bottom,
            );
            overlay.show();
        }

        let Some(mut surface_overlay) = self.water_surface_overlay.clone() else {
            return;
        };

        if mask.show_surface_line {
            set_polygon_rect(
                &mut surface_overlay,
                -PLAYER_HALF_WIDTH_PX,
                PLAYER_HALF_WIDTH_PX,
                top,
                (top + WATER_SURFACE_OVERLAY_HEIGHT_PX).min(bottom),
            );
            surface_overlay.show();
        } else {
            surface_overlay.hide();
        }
    }

    fn hide_water_overlay(&mut self) {
        if let Some(mut overlay) = self.water_body_overlay.clone() {
            overlay.hide();
        }
        if let Some(mut overlay) = self.water_surface_overlay.clone() {
            overlay.hide();
        }
    }

    fn start_climbing(&mut self, body: &mut Gd<CharacterBody2D>) {
        self.is_climbing = true;
        self.set_aim_indicator_visible(false);
        self.hide_water_overlay();
        self.drop_controller.reset(body);
        if let Some(movement) = &mut self.movement {
            movement.reset_transient_state();
        }
    }

    fn stop_climbing(&mut self) {
        self.is_climbing = false;
        if let Some(movement) = &mut self.movement {
            movement.reset_transient_state();
        }
    }

    fn physics_process_climb(&mut self, movement_input: MovementInput) {
        self.set_aim_indicator_visible(false);
        self.hide_water_overlay();

        let climb_velocity = self.movement.as_ref().map_or(Vector2::ZERO, |movement| {
            movement.climb_velocity(movement_input)
        });

        self.base_mut().set_velocity(climb_velocity);
        self.base_mut().move_and_slide();

        if self.check_hazard_collision() {
            self.start_death();
            return;
        }

        self.sprite.set_scale(Vector2::new(1.0, 1.0));
        animation::set_animation_paused(
            &mut self.sprite,
            self.animation_names.climb,
            climb_velocity.is_zero_approx(),
        );

        if !self.is_touching_ladder() {
            self.stop_climbing();
        }
    }

    fn is_touching_ladder(&self) -> bool {
        let player = self.to_gd().upcast::<Node2D>();
        ladder::is_touching_ladder(&player)
    }

    fn update_ladder_regrab_block(&mut self, movement_input: MovementInput, touching_ladder: bool) {
        if !self.ladder_regrab_blocked {
            return;
        }

        if ladder::should_clear_regrab_block(movement_input, touching_ladder) {
            self.ladder_regrab_blocked = false;
        }
    }

    fn resolve_water_contact(&self, velocity: Vector2, snap_depth: f32) -> PlayerWaterContact {
        let player = self.to_gd().upcast::<Node2D>();
        let player_position = player.get_global_position();
        let tree = player.get_tree();
        let water_zones = tree.get_nodes_in_group(WATER_ZONE_GROUP);
        let mut zones = Vec::new();
        let mut bounds = Vec::new();

        for node in water_zones.iter_shared() {
            let Ok(water_zone) = node.try_cast::<WaterZone>() else {
                continue;
            };

            let water_position = water_zone.clone().upcast::<Node2D>().get_global_position();
            let water_size = water_zone.bind().water_size();
            let index = zones.len();
            zones.push(water_zone);
            bounds.push((
                index,
                water::WaterBounds::from_center_size(water_position, water_size),
            ));
        }

        let resolved =
            water::resolve_targeted_contact(player_position, velocity, bounds, snap_depth);
        let zone = resolved
            .zone_index
            .and_then(|index| zones.get(index).cloned());

        PlayerWaterContact {
            contact: resolved.contact,
            zone,
        }
    }

    fn snap_to_water_surface_float(
        &mut self,
        body: &mut Gd<CharacterBody2D>,
        surface_y: f32,
        tuning: water::WaterTuning,
    ) {
        let mut position = body.get_global_position();
        position.y = water::surface_float_center_y(surface_y, tuning);
        body.set_global_position(position);
    }

    fn start_death(&mut self) {
        if self.is_dying {
            return;
        }
        self.is_climbing = false;
        self.ladder_regrab_blocked = false;
        self.water_state.update(water::WaterContact::None);
        self.last_water_zone = None;
        self.is_dying = true;
        self.set_aim_indicator_visible(false);
        self.hide_water_overlay();
        self.base_mut().set_velocity(Vector2::ZERO);
        self.sprite.set_animation(DEATH_ANIMATION);
        self.sprite.set_frame(0);
        self.sprite.play();
    }

    fn update_aim_indicator(&mut self, input: AimInput) {
        let visual = aim_indicator::resolve_indicator_visual(
            self.aim_direction,
            input,
            self.aim_indicator_distance,
        );
        self.aim_direction = visual.facing;

        let Some(indicator) = self.aim_indicator.as_mut() else {
            return;
        };
        indicator.bind_mut().apply_visual(visual);
    }

    fn set_aim_indicator_visible(&mut self, visible: bool) {
        let Some(indicator) = self.aim_indicator.as_ref() else {
            return;
        };
        indicator.clone().bind_mut().set_indicator_visible(visible);
    }

    fn check_hazard_collision(&mut self) -> bool {
        let collision_count = self.base().get_slide_collision_count();
        for index in 0..collision_count {
            let Some(collision) = self.base_mut().get_slide_collision(index) else {
                continue;
            };
            if hazard::is_hazard_collision(&collision, HAZARD_LAYER, &HAZARD_TILEMAP_PREFIXES) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exported_water_tuning_values_feed_runtime_tuning() {
        let tuning = water_tuning_from_exports(10.0, 6.0, 0.95, 0.55, -35.0, -80.0, 60.0);

        assert_eq!(tuning.surface_snap_depth, 10.0);
        assert_eq!(tuning.surface_float_depth, 6.0);
        assert_eq!(tuning.surface_horizontal_speed_multiplier, 0.95);
        assert_eq!(tuning.submerged_horizontal_speed_multiplier, 0.55);
        assert_eq!(tuning.buoyancy_velocity, -35.0);
        assert_eq!(tuning.swim_rise_velocity, -80.0);
        assert_eq!(tuning.swim_descend_velocity, 60.0);
    }
}
