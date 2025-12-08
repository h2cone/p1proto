use godot::classes::{AnimatedSprite2D, Area2D, CharacterBody2D, IArea2D};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Area2D)]
pub struct Checkpoint {
    #[base]
    base: Base<Area2D>,

    /// Has this checkpoint been activated?
    activated: bool,

    /// AnimatedSprite2D node reference
    sprite: Option<Gd<AnimatedSprite2D>>,
}

#[godot_api]
impl IArea2D for Checkpoint {
    fn init(base: Base<Area2D>) -> Self {
        Self {
            base,
            activated: false,
            sprite: None,
        }
    }

    fn ready(&mut self) {
        // Get reference to AnimatedSprite2D child
        if let Some(mut sprite) = self
            .base()
            .try_get_node_as::<AnimatedSprite2D>("AnimatedSprite2D")
        {
            self.sprite = Some(sprite.clone());

            // Start with unchecked animation
            sprite.play_ex().name("unchecked").done();
        }

        // Connect body_entered signal
        let callable = self.base().callable("on_body_entered");
        self.base_mut().connect("body_entered", &callable);
    }
}

#[godot_api]
impl Checkpoint {
    /// Called when a body enters the checkpoint area
    #[func]
    fn on_body_entered(&mut self, body: Gd<Node2D>) {
        // Only activate once
        if self.activated {
            return;
        }

        // Check if the body is the player (layer 1 = "player")
        if let Ok(body) = body.try_cast::<CharacterBody2D>() {
            let collision_layer = body.get_collision_layer();

            // Check if player layer (bit 0) is set
            if collision_layer & 1 != 0 {
                self.activate();
            }
        }
    }

    /// Activate the checkpoint
    #[func]
    fn activate(&mut self) {
        if self.activated {
            return;
        }

        self.activated = true;
        godot_print!("Checkpoint activated!");

        // Play raising animation
        if let Some(sprite) = self.sprite.as_mut() {
            sprite.play_ex().name("raising").done();
        }

        // TODO: Save game state to checkpoint
        // This could involve:
        // - Saving player position
        // - Saving checkpoint ID
        // - Triggering autosave
    }

    /// Check if checkpoint has been activated
    #[func]
    fn is_activated(&self) -> bool {
        self.activated
    }

    /// Reset checkpoint to unchecked state (for testing/debugging)
    #[func]
    fn reset(&mut self) {
        self.activated = false;

        if let Some(sprite) = self.sprite.as_mut() {
            sprite.play_ex().name("unchecked").done();
        }
    }
}
