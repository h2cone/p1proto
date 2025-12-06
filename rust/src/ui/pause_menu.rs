use godot::{
    classes::{Control, IControl},
    prelude::*,
};

/// PauseMenu manages the game's pause menu UI and handles pause/resume functionality
#[derive(GodotClass)]
#[class(base=Control)]
pub struct PauseMenu {
    base: Base<Control>,
}

#[godot_api]
impl IControl for PauseMenu {
    fn init(base: Base<Control>) -> Self {
        Self { base }
    }

    fn ready(&mut self) {
        godot_print!("PauseMenu ready");

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
            self.toggle_pause();
        }
    }
}

#[godot_api]
impl PauseMenu {
    /// Connect signals from UI buttons to handler methods
    fn connect_button_signals(&mut self) {
        // Connect resume button
        if let Some(mut resume_button) = self
            .base()
            .try_get_node_as::<godot::classes::Button>("VBoxContainer/ResumeButton")
        {
            let callable = self.base().callable("on_resume_button_pressed");
            resume_button.connect("pressed", &callable);
        } else {
            godot_error!("ResumeButton not found in PauseMenu scene");
        }

        // Connect quit button
        if let Some(mut quit_button) = self
            .base()
            .try_get_node_as::<godot::classes::Button>("VBoxContainer/QuitButton")
        {
            let callable = self.base().callable("on_quit_button_pressed");
            quit_button.connect("pressed", &callable);
        } else {
            godot_error!("QuitButton not found in PauseMenu scene");
        }
    }

    /// Toggle pause state - show/hide menu and pause/unpause game
    fn toggle_pause(&mut self) {
        let is_visible = self.base().is_visible();
        let new_state = !is_visible;

        // Toggle visibility
        self.base_mut().set_visible(new_state);

        // Toggle pause state
        if let Some(mut tree) = self.base().get_tree() {
            tree.set_pause(new_state);
        }

        godot_print!("Pause toggled: {}", new_state);
    }

    /// Handle resume button press - unpause and hide menu
    #[func]
    fn on_resume_button_pressed(&mut self) {
        godot_print!("Resume button pressed");
        self.toggle_pause();
    }

    /// Handle quit button press - return to main menu
    #[func]
    fn on_quit_button_pressed(&mut self) {
        godot_print!("Quit to menu button pressed");

        // Unpause before changing scene
        if let Some(mut tree) = self.base().get_tree() {
            tree.set_pause(false);
            let _result = tree.change_scene_to_file("res://ui/main_menu.tscn");
        }
    }
}
