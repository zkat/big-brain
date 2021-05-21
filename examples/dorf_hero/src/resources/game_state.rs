use std::collections::HashSet;

use bevy_tilemap::{Tile, Tilemap};

use crate::components::{Position, Render};

#[derive(Default, Clone)]
pub struct GameState {
    pub spawned: bool,
    pub walls: HashSet<(i32, i32)>,
}

impl GameState {
    pub fn try_move(
        &mut self,
        map: &mut Tilemap,
        render: &Render,
        position: &mut Position,
        delta_xy: (i32, i32),
    ) -> bool {
        let prev_pos = *position;
        let new_pos = (position.x + delta_xy.0, position.y + delta_xy.1);
        if !self.walls.contains(&new_pos) {
            position.x += delta_xy.0;
            position.y += delta_xy.1;
            move_sprite(map, prev_pos, *position, render);
            true
        } else {
            false
        }
    }
}


fn move_sprite(
    map: &mut Tilemap,
    previous_position: Position,
    position: Position,
    render: &Render,
) {
    // We need to first remove where we were prior.
    map.clear_tile((previous_position.x, previous_position.y), 1)
        .unwrap();
    // We then need to update where we are going!
    let tile = Tile {
        point: (position.x, position.y),
        sprite_index: render.sprite_index,
        sprite_order: render.sprite_order,
        ..Default::default()
    };
    map.insert_tile(tile).unwrap();
}
