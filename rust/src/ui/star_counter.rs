use godot::classes::{Control, IControl, Label};
use godot::prelude::*;

use crate::save;

/// StarCounter displays the number of collected stars in the UI.
/// Updates automatically when stars are collected.
#[derive(GodotClass)]
#[class(base=Control)]
pub struct StarCounter {
    base: Base<Control>,

    label: OnReady<Gd<Label>>,

    /// Cached count to detect changes
    cached_count: usize,
}

#[godot_api]
impl IControl for StarCounter {
    fn init(base: Base<Control>) -> Self {
        Self {
            base,
            label: OnReady::from_node("Label"),
            cached_count: 0,
        }
    }

    fn ready(&mut self) {
        self.update_display();
    }

    fn process(&mut self, _delta: f64) {
        let current = save::get_star_count();
        if current != self.cached_count {
            self.cached_count = current;
            self.update_display();
        }
    }
}

#[godot_api]
impl StarCounter {
    fn update_display(&mut self) {
        self.label.set_text(&format!("{}", self.cached_count));
    }

    /// Get current star count (callable from GDScript).
    #[func]
    pub fn get_count(&self) -> i64 {
        save::get_star_count() as i64
    }
}
