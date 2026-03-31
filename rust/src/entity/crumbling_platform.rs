use godot::classes::{AnimatableBody2D, AnimatedSprite2D, Area2D, IAnimatableBody2D, Node2D};
use godot::prelude::*;

/// Collision layer for crumbling platforms (layer 13, value 4096).
const CRUMBLING_PLATFORM_LAYER: i32 = 13;

/// State machine for crumbling platform behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum CrumbleState {
    /// Platform is stable, waiting for player contact.
    #[default]
    Idle,
    /// Player stepped on, platform is shaking with cracks appearing.
    Shaking,
    /// Platform is breaking apart and falling.
    Crumbling,
    /// Platform has crumbled, waiting to respawn (if enabled).
    Fallen,
}

/// A platform that crumbles after the player steps on it.
///
/// Behavior:
/// 1. Player lands on platform -> starts shake timer
/// 2. After shake_delay, plays "shake" animation
/// 3. After shake animation, plays "crumble" animation and disables collision
/// 4. Optionally respawns after respawn_time
#[derive(GodotClass)]
#[class(base=AnimatableBody2D)]
pub struct CrumblingPlatform {
    #[base]
    base: Base<AnimatableBody2D>,

    /// Seconds before shaking starts after player contact.
    #[export]
    shake_delay: f64,

    /// Seconds before platform respawns (0 = no respawn).
    #[export]
    respawn_time: f64,

    /// Current state of the platform.
    state: CrumbleState,

    /// Timer for state transitions.
    timer: f64,

    /// AnimatedSprite2D node reference.
    sprite: OnReady<Gd<AnimatedSprite2D>>,

    /// Area2D for detecting player landing.
    landing_detector: OnReady<Gd<Area2D>>,

    /// Whether a body is currently on the platform.
    body_on_platform: bool,
}

#[godot_api]
impl IAnimatableBody2D for CrumblingPlatform {
    fn init(base: Base<AnimatableBody2D>) -> Self {
        Self {
            base,
            shake_delay: 0.3,
            respawn_time: 3.0,
            state: CrumbleState::default(),
            timer: 0.0,
            sprite: OnReady::from_node("AnimatedSprite2D"),
            landing_detector: OnReady::from_node("LandingDetector"),
            body_on_platform: false,
        }
    }

    fn ready(&mut self) {
        // Ensure animations don't loop so animation_finished fires
        if let Some(mut frames) = self.sprite.get_sprite_frames() {
            frames.set_animation_loop("shake", false);
            frames.set_animation_loop("crumble", false);
        }

        self.sprite.set_animation("idle");
        self.sprite.stop();

        let platform = self.to_gd();

        // Connect animation_finished signal
        self.sprite
            .signals()
            .animation_finished()
            .connect_other(&platform, Self::on_animation_finished);

        // Connect landing detector signals
        self.landing_detector
            .signals()
            .body_entered()
            .connect_other(&platform, Self::on_body_entered);
        self.landing_detector
            .signals()
            .body_exited()
            .connect_other(&platform, Self::on_body_exited);
    }

    fn physics_process(&mut self, delta: f64) {
        match self.state {
            CrumbleState::Idle => {
                if self.body_on_platform {
                    self.timer += delta;
                    if self.timer >= self.shake_delay {
                        self.start_shaking();
                    }
                }
            }
            CrumbleState::Shaking | CrumbleState::Crumbling => {
                // Animation-driven, handled by on_animation_finished
            }
            CrumbleState::Fallen => {
                if self.respawn_time > 0.0 {
                    self.timer += delta;
                    if self.timer >= self.respawn_time {
                        self.respawn();
                    }
                }
            }
        }
    }
}

#[godot_api]
impl CrumblingPlatform {
    #[func]
    fn on_body_entered(&mut self, body: Gd<Node2D>) {
        if body.is_class("Player") {
            self.on_body_landed();
        }
    }

    #[func]
    fn on_body_exited(&mut self, body: Gd<Node2D>) {
        if body.is_class("Player") {
            self.on_body_left();
        }
    }

    fn on_body_landed(&mut self) {
        if self.state == CrumbleState::Idle && !self.body_on_platform {
            self.body_on_platform = true;
            self.timer = 0.0;
        }
    }

    fn on_body_left(&mut self) {
        self.body_on_platform = false;
        // Don't reset timer - once triggered, the platform will crumble
    }

    #[func]
    fn on_animation_finished(&mut self) {
        match self.state {
            CrumbleState::Shaking => {
                self.start_crumbling();
            }
            CrumbleState::Crumbling => {
                self.finish_crumble();
            }
            _ => {}
        }
    }

    fn start_shaking(&mut self) {
        self.state = CrumbleState::Shaking;
        self.sprite.set_animation("shake");
        self.sprite.play();
    }

    fn start_crumbling(&mut self) {
        self.state = CrumbleState::Crumbling;

        // Disable collision so player falls through
        self.base_mut()
            .set_collision_layer_value(CRUMBLING_PLATFORM_LAYER, false);

        self.sprite.set_animation("crumble");
        self.sprite.play();
    }

    fn finish_crumble(&mut self) {
        self.state = CrumbleState::Fallen;
        self.timer = 0.0;
        self.sprite.set_visible(false);
    }

    fn respawn(&mut self) {
        self.state = CrumbleState::Idle;
        self.timer = 0.0;
        self.body_on_platform = false;

        // Re-enable collision
        self.base_mut()
            .set_collision_layer_value(CRUMBLING_PLATFORM_LAYER, true);

        // Reset sprite
        self.sprite.set_visible(true);
        self.sprite.set_animation("idle");
        self.sprite.stop();
    }

    /// Check if the platform is currently solid (can be stood on).
    #[func]
    pub fn is_solid(&self) -> bool {
        matches!(self.state, CrumbleState::Idle | CrumbleState::Shaking)
    }

    #[func]
    pub fn get_state_name(&self) -> GString {
        match self.state {
            CrumbleState::Idle => "idle".into(),
            CrumbleState::Shaking => "shaking".into(),
            CrumbleState::Crumbling => "crumbling".into(),
            CrumbleState::Fallen => "fallen".into(),
        }
    }
}
