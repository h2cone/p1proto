use godot::{
    classes::{Button, Control, IControl},
    prelude::*,
};

/// MainMenu manages the game's main menu UI and handles button interactions
#[derive(GodotClass)]
#[class(base=Control)]
pub struct MainMenu {
    base: Base<Control>,
    play_button: OnReady<Gd<Button>>,
    quit_button: OnReady<Gd<Button>>,
}

#[godot_api]
impl IControl for MainMenu {
    fn init(base: Base<Control>) -> Self {
        Self {
            base,
            play_button: OnReady::from_node("VBoxContainer/PlayButton"),
            quit_button: OnReady::from_node("VBoxContainer/QuitButton"),
        }
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
        let main_menu = self.to_gd();

        self.play_button
            .signals()
            .pressed()
            .connect_other(&main_menu, Self::on_play_button_pressed);

        self.quit_button
            .signals()
            .pressed()
            .connect_other(&main_menu, Self::on_quit_button_pressed);
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
