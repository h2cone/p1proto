use godot::{
    classes::{CharacterBody2D, RigidBody2D},
    prelude::*,
};

pub fn push_rigid_bodies(body: &mut Gd<CharacterBody2D>, input_dir: f32, push_speed: f32) {
    if input_dir.abs() < 0.01 {
        return;
    }

    let collision_count = body.get_slide_collision_count();
    for index in 0..collision_count {
        let Some(collision) = body.get_slide_collision(index) else {
            continue;
        };
        let Some(collider) = collision.get_collider() else {
            continue;
        };
        if let Ok(mut rigid_body) = collider.try_cast::<RigidBody2D>() {
            let crate_vel = rigid_body.get_linear_velocity();
            rigid_body.set_linear_velocity(Vector2::new(input_dir * push_speed, crate_vel.y));

            if crate_vel.x.abs() < 1.0 {
                let current_pos = rigid_body.get_global_position();
                let push_delta = input_dir.signum() * 0.5;
                rigid_body
                    .set_global_position(Vector2::new(current_pos.x + push_delta, current_pos.y));
            }
        }
    }
}
