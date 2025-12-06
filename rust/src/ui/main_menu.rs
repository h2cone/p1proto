use godot::{
    classes::{Control, IControl},
    prelude::*,
};

/// MainMenu manages the game's main menu UI and handles button interactions
#[derive(GodotClass)]
#[class(base=Control)]
pub struct MainMenu {
    base: Base<Control>,
}

#[godot_api]
impl IControl for MainMenu {
    fn init(base: Base<Control>) -> Self {
        Self { base }
    }

    fn ready(&mut self) {
        godot_print!("MainMenu ready");

        // Connect button signals
        self.connect_button_signals();
    }
}

#[godot_api]
impl MainMenu {
    /// Connect signals from UI buttons to handler methods
    fn connect_button_signals(&mut self) {
        // Connect play button
        if let Some(mut new_game_button) = self
            .base()
            .try_get_node_as::<godot::classes::Button>("VBoxContainer/PlayButton")
        {
            let callable = self.base().callable("on_play_button_pressed");
            new_game_button.connect("pressed", &callable);
        } else {
            godot_error!("PlayButton not found in MainMenu scene");
        }

        // Connect quit button
        if let Some(mut quit_button) = self
            .base()
            .try_get_node_as::<godot::classes::Button>("VBoxContainer/QuitButton")
        {
            let callable = self.base().callable("on_quit_button_pressed");
            quit_button.connect("pressed", &callable);
        } else {
            godot_error!("QuitButton not found in MainMenu scene");
        }
    }

    /// Handle play button press - load the game scene
    #[func]
    fn on_play_button_pressed(&mut self) {
        godot_print!("Play button pressed");

        // Load and switch to game scene
        if let Some(mut tree) = self.base().get_tree() {
            let _result = tree.change_scene_to_file("res://game.tscn");
        }
    }

    /// Handle quit button press - exit the application
    #[func]
    fn on_quit_button_pressed(&mut self) {
        godot_print!("Quit button pressed");

        // Quit the application
        if let Some(mut tree) = self.base().get_tree() {
            tree.quit();
        }
    }
}
