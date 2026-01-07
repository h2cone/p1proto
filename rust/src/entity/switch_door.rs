use godot::classes::{AnimatedSprite2D, CollisionShape2D, IStaticBody2D, StaticBody2D};
use godot::prelude::*;

/// Door states for the switch door state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DoorState {
    Closed,
    Opening,
    Open,
    Closing,
}

/// A door that can be opened/closed by external triggers (e.g., pressure plates).
/// Blocks the player when closed and allows passage when open.
#[derive(GodotClass)]
#[class(base=StaticBody2D)]
pub struct SwitchDoor {
    #[base]
    base: Base<StaticBody2D>,

    state: DoorState,

    sprite: OnReady<Gd<AnimatedSprite2D>>,
    collision: OnReady<Gd<CollisionShape2D>>,

    /// Room coordinates
    #[export]
    room_coords: Vector2i,

    /// If true, the door starts in the open state
    #[export]
    starts_open: bool,
}

#[godot_api]
impl IStaticBody2D for SwitchDoor {
    fn init(base: Base<StaticBody2D>) -> Self {
        Self {
            base,
            state: DoorState::Closed,
            sprite: OnReady::from_node("AnimatedSprite2D"),
            collision: OnReady::from_node("CollisionShape2D"),
            room_coords: Vector2i::ZERO,
            starts_open: false,
        }
    }

    fn ready(&mut self) {
        // Connect AnimatedSprite2D's animation_finished signal
        let callable = self.base().callable("on_animation_finished");
        self.sprite.connect("animation_finished", &callable);

        // Ensure transition animations don't loop
        if let Some(mut frames) = self.sprite.get_sprite_frames() {
            frames.set_animation_loop("opening", false);
            frames.set_animation_loop("closing", false);
        }

        // Set initial state
        if self.starts_open {
            self.state = DoorState::Open;
            self.sprite.set_animation("open");
            self.collision.set_disabled(true);
        } else {
            self.state = DoorState::Closed;
            self.sprite.set_animation("closed");
            self.collision.set_disabled(false);
        }
        self.sprite.stop();
    }
}

#[godot_api]
impl SwitchDoor {
    /// Signal emitted when animation finishes (forwarded from AnimatedSprite2D)
    #[signal]
    fn animation_finished();

    /// Signal emitted when the door fully opens
    #[signal]
    fn door_opened();

    /// Signal emitted when the door fully closes
    #[signal]
    fn door_closed();

    /// Open the door (if not already open or opening)
    #[func]
    pub fn open(&mut self) {
        match self.state {
            DoorState::Closed | DoorState::Closing => {
                self.state = DoorState::Opening;
                self.sprite.set_animation("opening");
                self.sprite.set_frame(0);
                self.sprite.play();
            }
            _ => {}
        }
    }

    /// Close the door (if not already closed or closing)
    #[func]
    pub fn close(&mut self) {
        match self.state {
            DoorState::Open | DoorState::Opening => {
                self.state = DoorState::Closing;
                self.sprite.set_animation("closing");
                self.sprite.set_frame(0);
                self.sprite.play();
                // Re-enable collision when starting to close (deferred to avoid physics query conflicts)
                self.collision.set_deferred("disabled", &false.to_variant());
            }
            _ => {}
        }
    }

    /// Toggle the door state
    #[func]
    pub fn toggle(&mut self) {
        match self.state {
            DoorState::Closed | DoorState::Closing => self.open(),
            DoorState::Open | DoorState::Opening => self.close(),
        }
    }

    /// Check if the door is currently open
    #[func]
    pub fn is_open(&self) -> bool {
        self.state == DoorState::Open
    }

    /// Check if the door is currently closed
    #[func]
    pub fn is_closed(&self) -> bool {
        self.state == DoorState::Closed
    }

    /// Called when animation finishes
    #[func]
    fn on_animation_finished(&mut self) {
        match self.state {
            DoorState::Opening => {
                self.state = DoorState::Open;
                self.sprite.stop();
                self.sprite.set_animation("open");
                // Disable collision when fully open (deferred to avoid physics query conflicts)
                self.collision.set_deferred("disabled", &true.to_variant());
                self.signals().door_opened().emit();
            }
            DoorState::Closing => {
                self.state = DoorState::Closed;
                self.sprite.stop();
                self.sprite.set_animation("closed");
                self.signals().door_closed().emit();
            }
            _ => {}
        }
    }
}
