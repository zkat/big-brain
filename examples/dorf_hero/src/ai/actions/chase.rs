use bevy::prelude::*;
use bevy_tilemap::Tilemap;
use big_brain::prelude::*;

use crate::components::{Position, Render};
use crate::resources::GameState;

// Let's define our "default" action, which will be used whenever there's nothing in particular getting our dorf's attention.
#[derive(Default, Debug, Clone)]
pub struct ChaseBuilder;

#[derive(Debug, Default, Clone)]
pub struct Chase {
    
}

impl Chase {
    pub fn build() -> ChaseBuilder {
        ChaseBuilder
    }
}

impl ActionBuilder for ChaseBuilder {
    fn build(&self, cmd: &mut Commands, action: Entity, _actor: Entity) {
        cmd.entity(action).insert(Chase::default());
    }
}

pub fn meander_action(
    mut game_state: ResMut<GameState>,
    time: Res<Time>,
    mut map_query: Query<(&mut Tilemap, &mut Timer)>,
    mut location_query: Query<(&mut Position, &Render)>,
    mut action_q: Query<(&mut Chase, &Actor, &mut ActionState)>,
) {
    for (mut map, mut timer) in map_query.iter_mut() {
        timer.tick(time.delta());
        if !timer.finished() {
            continue;
        }
        for (mut chase, Actor(actor), mut state) in action_q.iter_mut() {
            match *state {
                ActionState::Cancelled => {
                    // *ALWAYS* handle Cancelled, even if you're doing a single-tick action. Your action might end up hanging otherwise.
                    *state = ActionState::Success;
                    continue;
                }
                ActionState::Init | ActionState::Success | ActionState::Failure => {
                    continue;
                }
                // These two fall through to logic :)
                ActionState::Requested => {
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {}
            }
            if let Ok((mut pos, render)) = location_query.get_mut(*actor) {
                if (chase.dx == 0 && chase.dy == 0)
                    || !game_state.try_move(&mut *map, render, &mut pos, (chase.dx, chase.dy))
                {
                    let mut rng = rand::thread_rng();
                    chase.dx = rng.gen_range(-1..=1);
                    chase.dy = rng.gen_range(-1..=1);
                }
            }
        }
    }
}

