mod aim_indicator;
mod animation;
mod hazard;
mod input_adapter;
mod ladder;
mod platform;
mod push;

pub use crate::core::player::{MovementConfig, MovementInput, MovementState, PlayerMovement};
pub use animation::AnimationNames;
pub use input_adapter::InputActions;

use godot::{
    classes::{AnimatedSprite2D, CharacterBody2D, ICharacterBody2D, Node2D, ProjectSettings},
    prelude::*,
};

use self::aim_indicator::{AimDirection, AimIndicator, AimInput};
use self::platform::PlatformDropController;

const MOVING_PLATFORM_LAYER: i32 = 4;
const HAZARD_LAYER: i32 = 12;
const DROP_THROUGH_DURATION: f64 = 0.35;
const PUSH_SPEED: f32 = 80.0;
const DEATH_ANIMATION: &str = "death";
const CLIMB_START_THRESHOLD: f32 = -0.2;
const HAZARD_TILEMAP_PREFIXES: [&str; 2] = ["HazardsTiles", "Hazards"];

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Player {
    base: Base<CharacterBody2D>,
    movement: Option<PlayerMovement>,
    sprite: OnReady<Gd<AnimatedSprite2D>>,
    aim_indicator: Option<Gd<AimIndicator>>,
    #[export]
    aim_indicator_distance: f32,
    input_actions: InputActions,
    animation_names: AnimationNames,
    drop_controller: PlatformDropController,
    aim_direction: AimDirection,
    is_dying: bool,
    is_climbing: bool,
    ladder_regrab_blocked: bool,
}

#[godot_api]
impl ICharacterBody2D for Player {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            base,
            movement: None,
            sprite: OnReady::from_node("AnimatedSprite2D"),
            aim_indicator: None,
            aim_indicator_distance: 12.0,
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

        godot_print!("[Player] ready")
    }

    fn physics_process(&mut self, delta: f64) {
        if self.is_dying {
            self.base_mut().set_velocity(Vector2::ZERO);
            self.set_aim_indicator_visible(false);
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

        if !jumped_from_ladder
            && !self.ladder_regrab_blocked
            && movement_input.vertical_direction <= CLIMB_START_THRESHOLD
            && touching_ladder
        {
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

        let aim_input = input_adapter::collect_aim_input(&self.input_actions);
        let Some(movement) = self.movement.as_mut() else {
            return;
        };
        let new_velocity = movement.physics_process(
            velocity,
            is_on_floor || jumped_from_ladder,
            delta,
            movement_input,
        );

        self.base_mut().set_velocity(new_velocity);
        self.base_mut().move_and_slide();

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

#[godot_api]
impl Player {
    #[signal]
    pub(crate) fn death_finished();

    #[func]
    fn on_animation_finished(&mut self) {
        if !self.is_dying {
            return;
        }

        if self.sprite.get_animation() == StringName::from(DEATH_ANIMATION) {
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

        let mut body = self.to_gd().upcast::<CharacterBody2D>();
        self.drop_controller.reset(&mut body);
        self.set_aim_indicator_visible(false);
    }

    fn start_climbing(&mut self, body: &mut Gd<CharacterBody2D>) {
        self.is_climbing = true;
        self.set_aim_indicator_visible(false);
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

        if !touching_ladder || movement_input.vertical_direction > CLIMB_START_THRESHOLD {
            self.ladder_regrab_blocked = false;
        }
    }

    fn start_death(&mut self) {
        if self.is_dying {
            return;
        }
        self.is_climbing = false;
        self.ladder_regrab_blocked = false;
        self.is_dying = true;
        self.set_aim_indicator_visible(false);
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
