use godot::{
    classes::{CharacterBody2D, CollisionObject2D},
    prelude::*,
};

pub struct PlatformDropController {
    timer: f64,
    duration: f64,
    collision_layer: i32,
    mask_default: bool,
}

impl PlatformDropController {
    pub fn new(duration: f64, collision_layer: i32) -> Self {
        Self {
            timer: 0.0,
            duration,
            collision_layer,
            mask_default: true,
        }
    }

    pub fn configure_mask_default(&mut self, mask_default: bool) {
        self.mask_default = mask_default;
    }

    pub fn update(
        &mut self,
        body: &mut Gd<CharacterBody2D>,
        is_on_floor: bool,
        delta: f64,
        drop_pressed: bool,
    ) {
        if is_on_floor
            && self.timer <= 0.0
            && drop_pressed
            && is_standing_on_platform(body, self.collision_layer)
        {
            self.start(body);
        }

        if self.timer > 0.0 {
            self.timer -= delta;
            if self.timer <= 0.0 {
                self.stop(body);
            }
        }
    }

    pub fn is_active(&self) -> bool {
        self.timer > 0.0
    }

    pub fn reset(&mut self, body: &mut Gd<CharacterBody2D>) {
        if self.timer > 0.0 {
            self.stop(body);
        }
    }

    fn start(&mut self, body: &mut Gd<CharacterBody2D>) {
        self.timer = self.duration;
        body.set_collision_mask_value(self.collision_layer, false);
    }

    fn stop(&mut self, body: &mut Gd<CharacterBody2D>) {
        self.timer = 0.0;
        body.set_collision_mask_value(self.collision_layer, self.mask_default);
    }
}

fn is_standing_on_platform(body: &mut Gd<CharacterBody2D>, collision_layer: i32) -> bool {
    let Some(collision) = body.get_last_slide_collision() else {
        return false;
    };

    let normal = collision.get_normal();
    let is_floor_hit = normal.dot(Vector2::new(0.0, -1.0)) > 0.7;
    if !is_floor_hit {
        return false;
    }

    let Some(collider) = collision.get_collider() else {
        return false;
    };

    if let Ok(body) = collider.try_cast::<CollisionObject2D>() {
        body.get_collision_layer_value(collision_layer)
    } else {
        false
    }
}
