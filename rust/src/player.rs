use godot::{
    classes::{CharacterBody2D, ICharacterBody2D, ProjectSettings},
    prelude::*,
};

use crate::player_movement::PlayerMovement;

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Player {
    base: Base<CharacterBody2D>,
    movement: Option<PlayerMovement>,
}

#[godot_api]
impl ICharacterBody2D for Player {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            base,
            movement: None,
        }
    }

    fn ready(&mut self) {
        let settings = ProjectSettings::singleton();
        let gravity = settings.get("physics/2d/default_gravity").to::<f64>() as f32;
        self.movement = Some(PlayerMovement::new(gravity));
        godot_print!("Player ready")
    }

    fn physics_process(&mut self, delta: f64) {
        // Get immutable values first
        let velocity = self.base().get_velocity();
        let is_on_floor = self.base().is_on_floor();

        // Process movement
        if let Some(movement) = &mut self.movement {
            let new_velocity = movement.physics_process(velocity, is_on_floor, delta);

            self.base_mut().set_velocity(new_velocity);
            self.base_mut().move_and_slide();
        }
    }
}
