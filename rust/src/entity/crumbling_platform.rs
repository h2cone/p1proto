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

/// Tracks whether crumble countdown has been armed by player contact.
#[derive(Debug, Clone, Copy, Default)]
struct CrumbleTrigger {
    body_on_platform: bool,
    countdown_started: bool,
}

impl CrumbleTrigger {
    fn on_body_landed(&mut self, state: CrumbleState, timer: &mut f64) {
        if state != CrumbleState::Idle {
            return;
        }

        self.body_on_platform = true;
        if !self.countdown_started {
            self.countdown_started = true;
            *timer = 0.0;
        }
    }

    fn on_body_left(&mut self) {
        self.body_on_platform = false;
    }

    fn advance(&self, timer: &mut f64, delta: f64, shake_delay: f64) -> bool {
        if !self.countdown_started {
            return false;
        }

        *timer += delta;
        *timer >= shake_delay
    }

    fn reset(&mut self) {
        *self = Self::default();
    }
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

    /// Whether crumble countdown has been armed by player contact.
    trigger: CrumbleTrigger,
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
            trigger: CrumbleTrigger::default(),
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
                if self
                    .trigger
                    .advance(&mut self.timer, delta, self.shake_delay)
                {
                    self.start_shaking();
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
        self.trigger.on_body_landed(self.state, &mut self.timer);
    }

    fn on_body_left(&mut self) {
        self.trigger.on_body_left();
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
        self.trigger.reset();

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

#[cfg(test)]
mod tests {
    use super::{CrumbleState, CrumbleTrigger};

    #[test]
    fn leaving_before_delay_does_not_cancel_crumble() {
        let mut trigger = CrumbleTrigger::default();
        let mut timer = 99.0;

        trigger.on_body_landed(CrumbleState::Idle, &mut timer);
        assert!(trigger.body_on_platform);
        assert!(trigger.countdown_started);
        assert_eq!(timer, 0.0);

        trigger.on_body_left();
        assert!(!trigger.body_on_platform);

        assert!(!trigger.advance(&mut timer, 0.29, 0.3));
        assert!(trigger.advance(&mut timer, 0.01, 0.3));
    }

    #[test]
    fn reentering_does_not_restart_locked_countdown() {
        let mut trigger = CrumbleTrigger::default();
        let mut timer = 0.0;

        trigger.on_body_landed(CrumbleState::Idle, &mut timer);
        assert!(!trigger.advance(&mut timer, 0.2, 0.3));

        trigger.on_body_left();
        trigger.on_body_landed(CrumbleState::Idle, &mut timer);
        assert_eq!(timer, 0.2);
        assert!(trigger.advance(&mut timer, 0.1, 0.3));
    }
}
