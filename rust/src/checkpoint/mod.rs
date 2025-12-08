use godot::classes::{AnimatedSprite2D, Area2D, IArea2D};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Area2D)]
pub struct Checkpoint {
    #[base]
    base: Base<Area2D>,

    /// Has this checkpoint been activated?
    activated: bool,

    /// AnimatedSprite2D node reference
    sprite: OnReady<Gd<AnimatedSprite2D>>,
}

#[godot_api]
impl IArea2D for Checkpoint {
    fn init(base: Base<Area2D>) -> Self {
        Self {
            base,
            activated: false,
            sprite: OnReady::from_node("AnimatedSprite2D"),
        }
    }

    fn ready(&mut self) {
        // Show the unchecked frame without autoplaying the animation
        self.sprite.set_animation("unchecked");
        self.sprite.stop();

        // Connect body_entered signal without string literals
        self.signals()
            .body_entered()
            .connect_self(Self::on_body_entered);
    }
}

#[godot_api]
impl Checkpoint {
    /// Called when a body enters the checkpoint area
    #[func]
    fn on_body_entered(&mut self, _body: Gd<Node2D>) {
        // Only activate once
        if self.activated {
            return;
        }
        self.activate();
    }

    /// Activate the checkpoint
    #[func]
    fn activate(&mut self) {
        if self.activated {
            return;
        }

        self.activated = true;
        godot_print!("Checkpoint activated!");

        // Immediately switch to checked loop
        self.sprite.set_animation("checked");
        self.sprite.play();

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
        self.sprite.set_animation("unchecked");
        self.sprite.stop();
    }
}
