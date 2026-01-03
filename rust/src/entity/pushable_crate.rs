use godot::classes::{IRigidBody2D, RigidBody2D};
use godot::prelude::*;

/// A crate that can be pushed by the player.
///
/// Uses RigidBody2D for physics-based movement. The crate responds to
/// player collisions naturally through Godot's physics engine.
///
/// ## Scene Setup
/// The scene should have the following structure:
/// - PushableCrate (RigidBody2D, script: pushable_crate.rs)
///   - CollisionShape2D (with appropriate shape)
///   - Sprite2D or AnimatedSprite2D
///
/// ## Recommended RigidBody2D Settings
/// - Gravity Scale: 1.0 (or adjust for desired fall speed)
/// - Linear Damp: 5.0-10.0 (prevents sliding too far)
/// - Lock Rotation: true (keeps crate upright)
/// - Freeze Mode: Kinematic (when frozen)
/// - Collision Layer: dedicated layer for crates
/// - Collision Mask: tiles, player, other crates
#[derive(GodotClass)]
#[class(base=RigidBody2D)]
pub struct PushableCrate {
    #[base]
    base: Base<RigidBody2D>,

    /// Room coordinates for save system integration.
    #[export]
    room_coords: Vector2i,

    /// Maximum horizontal push speed.
    #[export]
    max_push_speed: f32,

    /// Whether this crate's position should be saved.
    #[export]
    persist_position: bool,
}

#[godot_api]
impl IRigidBody2D for PushableCrate {
    fn init(base: Base<RigidBody2D>) -> Self {
        Self {
            base,
            room_coords: Vector2i::ZERO,
            max_push_speed: 50.0,
            persist_position: false,
        }
    }

    fn ready(&mut self) {
        godot_print!(
            "[PushableCrate] ready at {:?}, room {:?}",
            self.base().get_global_position(),
            self.room_coords
        );

        // Optionally restore saved position here if persist_position is enabled
        // if self.persist_position {
        //     if let Some(saved_pos) = self.load_saved_position() {
        //         self.base_mut().set_global_position(saved_pos);
        //     }
        // }
    }

    fn physics_process(&mut self, _delta: f64) {
        // Clamp horizontal velocity to prevent excessive speed
        let mut velocity = self.base().get_linear_velocity();
        velocity.x = velocity.x.clamp(-self.max_push_speed, self.max_push_speed);
        self.base_mut().set_linear_velocity(velocity);
    }
}

#[godot_api]
impl PushableCrate {
    /// Signal emitted when the crate lands on the ground.
    #[signal]
    fn landed();

    /// Signal emitted when the crate is pushed.
    #[signal]
    fn pushed();

    /// Freeze the crate in place (stops all physics simulation).
    #[func]
    fn freeze(&mut self) {
        self.base_mut().set_freeze_enabled(true);
    }

    /// Unfreeze the crate (resumes physics simulation).
    #[func]
    fn unfreeze(&mut self) {
        self.base_mut().set_freeze_enabled(false);
    }

    /// Check if the crate is currently frozen.
    #[func]
    fn is_frozen(&self) -> bool {
        self.base().is_freeze_enabled()
    }
}
