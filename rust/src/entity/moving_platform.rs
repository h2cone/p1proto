use godot::{
    builtin::Variant,
    classes::{AnimatableBody2D, IAnimatableBody2D, Object, Tween, tween},
    prelude::*,
};

/// Ping-ponging platform driven by a Tween.
#[derive(GodotClass)]
#[class(base=AnimatableBody2D)]
pub struct MovingPlatform {
    #[base]
    base: Base<AnimatableBody2D>,

    /// Offset from the start position to the far end of the movement.
    #[export]
    travel: Vector2,

    /// Seconds for one leg of the movement.
    #[export]
    duration: f64,

    /// Optional pause at each end of the path.
    #[export]
    pause_time: f64,

    start_position: Vector2,
    tween: Option<Gd<Tween>>,
}

#[godot_api]
impl IAnimatableBody2D for MovingPlatform {
    fn init(base: Base<AnimatableBody2D>) -> Self {
        Self {
            base,
            travel: Vector2::new(0.0, 0.0),
            duration: 0.0,
            pause_time: 0.0,
            start_position: Vector2::ZERO,
            tween: None,
        }
    }

    fn ready(&mut self) {
        self.start_position = self.base().get_position();
        self.start_motion();
    }
}

#[godot_api]
impl MovingPlatform {
    /// Restart the tween using the current position as the new start point.
    #[func]
    fn restart(&mut self) {
        self.start_position = self.base().get_position();
        self.start_motion();
    }

    fn start_motion(&mut self) {
        // Replace any previously running tween so only one controls the platform.
        if let Some(mut tween) = self.tween.take() {
            tween.kill();
        }

        let Some(mut tween) = self.base_mut().create_tween() else {
            godot_error!("MovingPlatform failed to create tween");
            return;
        };

        let _ = tween.set_process_mode(tween::TweenProcessMode::PHYSICS);
        let _ = tween.set_pause_mode(tween::TweenPauseMode::PROCESS);
        let _ = tween.set_trans(tween::TransitionType::SINE);
        let _ = tween.set_ease(tween::EaseType::IN_OUT);
        let _ = tween.set_loops_ex().loops(-1).done();

        let target_object: Gd<Object> = self.to_gd().upcast();
        let position_path = "position";

        let leg_duration = if self.duration <= 0.0 {
            0.001
        } else {
            self.duration
        };

        let end_position = self.start_position + self.travel;
        let start_variant = Variant::from(self.start_position);
        let end_variant = Variant::from(end_position);

        if tween
            .tween_property(&target_object, position_path, &end_variant, leg_duration)
            .is_none()
        {
            godot_warn!("MovingPlatform could not add forward tween");
        }

        if self.pause_time > 0.0 {
            let _ = tween.tween_interval(self.pause_time);
        }

        if tween
            .tween_property(&target_object, position_path, &start_variant, leg_duration)
            .is_none()
        {
            godot_warn!("MovingPlatform could not add return tween");
        }

        if self.pause_time > 0.0 {
            let _ = tween.tween_interval(self.pause_time);
        }

        self.tween = Some(tween);
    }
}
