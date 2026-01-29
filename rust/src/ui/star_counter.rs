use godot::classes::{Control, IControl, Label, Tween, tween};
use godot::prelude::*;

use crate::save;

/// How long the counter stays fully visible (seconds).
const DISPLAY_DURATION: f64 = 1.5;
/// Fade in/out duration (seconds).
const FADE_DURATION: f64 = 0.3;

/// StarCounter displays the number of collected stars in the UI.
/// Shows briefly when a star is collected, then fades out.
#[derive(GodotClass)]
#[class(base=Control)]
pub struct StarCounter {
    base: Base<Control>,

    label: OnReady<Gd<Label>>,

    /// Cached count to detect changes
    cached_count: usize,

    /// Current tween for fade animations
    current_tween: Option<Gd<Tween>>,
}

#[godot_api]
impl IControl for StarCounter {
    fn init(base: Base<Control>) -> Self {
        Self {
            base,
            label: OnReady::from_node("HBoxContainer/Label"),
            cached_count: 0,
            current_tween: None,
        }
    }

    fn ready(&mut self) {
        // Start hidden
        self.base_mut()
            .set_modulate(Color::from_rgba(1.0, 1.0, 1.0, 0.0));

        // Sync with current count without showing
        self.cached_count = save::get_star_count();
        self.update_display();
    }

    fn process(&mut self, _delta: f64) {
        let current = save::get_star_count();
        if current != self.cached_count {
            self.cached_count = current;
            self.update_display();
            self.show_briefly();
        }
    }
}

#[godot_api]
impl StarCounter {
    fn update_display(&mut self) {
        self.label.set_text(&format!("{}", self.cached_count));
    }

    fn show_briefly(&mut self) {
        // Kill any existing tween
        if let Some(mut tween) = self.current_tween.take() {
            tween.kill();
        }

        let Some(mut tween) = self.base().get_tree().and_then(|mut t| t.create_tween()) else {
            return;
        };

        tween.set_pause_mode(tween::TweenPauseMode::PROCESS);

        let target = self.to_gd();
        let visible = Color::from_rgba(1.0, 1.0, 1.0, 1.0);
        let hidden = Color::from_rgba(1.0, 1.0, 1.0, 0.0);

        // Fade in
        tween
            .tween_property(&target, "modulate", &visible.to_variant(), FADE_DURATION)
            .unwrap();

        // Wait
        tween.tween_interval(DISPLAY_DURATION);

        // Fade out
        tween
            .tween_property(&target, "modulate", &hidden.to_variant(), FADE_DURATION)
            .unwrap();

        self.current_tween = Some(tween);
    }

    /// Get current star count (callable from GDScript).
    #[func]
    pub fn get_count(&self) -> i64 {
        save::get_star_count() as i64
    }
}
