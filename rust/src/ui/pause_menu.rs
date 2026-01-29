use godot::{
    classes::{Button, Control, IControl, Label},
    prelude::*,
};

use crate::save;

/// PauseMenu manages the game's pause menu UI and handles pause/resume functionality
#[derive(GodotClass)]
#[class(base=Control)]
pub struct PauseMenu {
    base: Base<Control>,
    resume_button: OnReady<Gd<Button>>,
    quit_button: OnReady<Gd<Button>>,
    star_label: OnReady<Gd<Label>>,
}

#[godot_api]
impl IControl for PauseMenu {
    fn init(base: Base<Control>) -> Self {
        Self {
            base,
            resume_button: OnReady::from_node("VBoxContainer/ResumeButton"),
            quit_button: OnReady::from_node("VBoxContainer/QuitButton"),
            star_label: OnReady::from_node("StarDisplay/HBoxContainer/Label"),
        }
    }

    fn ready(&mut self) {
        godot_print!("[PauseMenu] ready");

        // Start hidden
        self.base_mut().set_visible(false);

        // Set pause mode to PROCESS so menu works when game is paused
        self.base_mut()
            .set_process_mode(godot::classes::node::ProcessMode::ALWAYS);

        // Connect button signals
        self.connect_button_signals();
    }

    fn process(&mut self, _delta: f64) {
        // Check for pause input
        let input = godot::classes::Input::singleton();

        if input.is_action_just_pressed("ui_esc") {
            if self.is_world_map_visible() {
                return;
            }
            self.toggle_pause();
        }
    }
}

#[godot_api]
impl PauseMenu {
    /// Connect signals from UI buttons to handler methods
    fn connect_button_signals(&mut self) {
        let pause_menu = self.to_gd();

        self.resume_button
            .signals()
            .pressed()
            .connect_other(&pause_menu, Self::on_resume_button_pressed);

        self.quit_button
            .signals()
            .pressed()
            .connect_other(&pause_menu, Self::on_quit_button_pressed);
    }

    /// Toggle pause state - show/hide menu and pause/unpause game
    fn toggle_pause(&mut self) {
        let is_visible = self.base().is_visible();
        let new_state = !is_visible;

        // Update star count when showing
        if new_state {
            self.update_star_display();
        }

        // Toggle visibility
        self.base_mut().set_visible(new_state);

        // Toggle pause state
        if let Some(mut tree) = self.base().get_tree() {
            tree.set_pause(new_state);
        }

        godot_print!("[PauseMenu] pause toggled: {}", new_state);
    }

    fn update_star_display(&mut self) {
        let count = save::get_star_count();
        self.star_label.set_text(&format!("{}", count));
    }

    fn is_world_map_visible(&self) -> bool {
        let Some(parent) = self.base().get_parent() else {
            return false;
        };
        let Some(world_map) = parent.get_node_or_null("WorldMap") else {
            return false;
        };
        let Ok(control) = world_map.try_cast::<Control>() else {
            return false;
        };
        control.is_visible()
    }

    /// Handle resume button press - unpause and hide menu
    #[func]
    fn on_resume_button_pressed(&mut self) {
        godot_print!("[PauseMenu] resume button pressed");
        self.toggle_pause();
    }

    /// Handle quit button press - return to main menu
    #[func]
    fn on_quit_button_pressed(&mut self) {
        godot_print!("[PauseMenu] quit to menu button pressed");

        // Unpause before changing scene
        if let Some(mut tree) = self.base().get_tree() {
            tree.set_pause(false);
            let _result = tree.change_scene_to_file("res://ui/main_menu.tscn");
        }
    }
}
