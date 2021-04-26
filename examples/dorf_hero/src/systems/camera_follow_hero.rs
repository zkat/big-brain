use bevy::{prelude::*, render::camera::Camera};

use crate::components::{Hero, Position};

pub fn camera_follow_hero(
    mut camera_q: Query<&mut Transform, With<Camera>>,
    player_q: Query<&Position, (With<Hero>, Changed<Position>)>,
) {
    if let (Ok(player_pos), Ok(mut camera_trans)) = (player_q.single(), camera_q.single_mut()) {
        camera_trans.translation.x = player_pos.x as f32 * 32.;
        camera_trans.translation.y = player_pos.y as f32 * 32.;
    }
}
