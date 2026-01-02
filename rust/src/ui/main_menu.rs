use godot::{
    classes::{Button, Control, IControl},
    prelude::*,
};

use crate::save::{self, DEFAULT_SAVE_SLOT};

/// MainMenu manages the game's main menu UI and handles button interactions
#[derive(GodotClass)]
#[class(base=Control)]
pub struct MainMenu {
    base: Base<Control>,
    play_button: OnReady<Gd<Button>>,
    continue_button: Option<Gd<Button>>,
    quit_button: OnReady<Gd<Button>>,
}

#[godot_api]
impl IControl for MainMenu {
    fn init(base: Base<Control>) -> Self {
        Self {
            base,
            play_button: OnReady::from_node("VBoxContainer/PlayButton"),
            continue_button: None,
            quit_button: OnReady::from_node("VBoxContainer/QuitButton"),
        }
    }

    fn ready(&mut self) {
        godot_print!("[MainMenu] ready");

        // Locate optional Continue button (Godot wiring handled in Rust)
        self.continue_button = self.find_continue_button();

        // Connect button signals
        self.connect_button_signals();

        // Update Continue button enabled state based on save availability
        self.update_continue_button_state();
    }
}

#[godot_api]
impl MainMenu {
    /// Connect signals from UI buttons to handler methods
    fn connect_button_signals(&mut self) {
        let main_menu = self.to_gd();

        self.play_button
            .signals()
            .pressed()
            .connect_other(&main_menu, Self::on_play_button_pressed);

        if let Some(button) = &self.continue_button {
            button
                .signals()
                .pressed()
                .connect_other(&main_menu, Self::on_continue_button_pressed);
        } else {
            godot_warn!("ContinueButton not found - continue flow will be unavailable");
        }

        self.quit_button
            .signals()
            .pressed()
            .connect_other(&main_menu, Self::on_quit_button_pressed);
    }

    /// Handle play button press - reset state and load the game scene
    #[func]
    fn on_play_button_pressed(&mut self) {
        godot_print!("[MainMenu] play button pressed");

        // Reset all game state for new game
        save::reset_all();

        // Load and switch to game scene
        if let Some(mut tree) = self.base().get_tree() {
            let _result = tree.change_scene_to_file("res://game.tscn");
        }
    }

    /// Handle continue button press - queue load and switch to game scene
    #[func]
    fn on_continue_button_pressed(&mut self) {
        if save::queue_load(DEFAULT_SAVE_SLOT) {
            godot_print!(
                "[MainMenu] continue button pressed - loading save slot {}",
                DEFAULT_SAVE_SLOT
            );
            if let Some(mut tree) = self.base().get_tree() {
                let _result = tree.change_scene_to_file("res://game.tscn");
            }
        } else {
            godot_warn!("Continue requested but no save data available");
        }
    }

    /// Expose whether the default save slot has data (for toggling UI state)
    #[func]
    fn has_checkpoint_save(&self) -> bool {
        save::has_save(DEFAULT_SAVE_SLOT)
    }

    /// Try to get the Continue button reference if it exists in the scene
    fn find_continue_button(&self) -> Option<Gd<Button>> {
        self.base()
            .try_get_node_as::<Button>("VBoxContainer/ContinueButton")
    }

    /// Enable/disable Continue based on whether a save exists
    fn update_continue_button_state(&mut self) {
        let has_save = self.has_checkpoint_save();

        if let Some(button) = self.continue_button.as_mut() {
            button.set_disabled(!has_save);
        }
    }

    /// Handle quit button press - exit the application
    #[func]
    fn on_quit_button_pressed(&mut self) {
        godot_print!("[MainMenu] quit button pressed");

        // Quit the application
        if let Some(mut tree) = self.base().get_tree() {
            tree.quit();
        }
    }
}
