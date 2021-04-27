use bevy::prelude::*;
use big_brain::{evaluators::{Evaluator, LinearEvaluator}, prelude::*};

use crate::components::{EvilDorf, Hero, Position};
use crate::util;

#[derive(Debug, Default)]
pub struct EnemyDistanceBuilder {
    within: f32,
}

impl EnemyDistanceBuilder {
    pub fn within(mut self, within: f32) -> Self {
        self.within = within;
        self
    }
}

impl ScorerBuilder for EnemyDistanceBuilder {
    fn build(&self, cmd: &mut Commands, scorer: Entity, _actor: Entity) {
        cmd.entity(scorer).insert(EnemyDistance {
            within: self.within,
            evaluator: LinearEvaluator::new_ranged(self.within, 0.0),
        });
    }
}

#[derive(Debug)]
pub struct EnemyDistance {
    within: f32,
    evaluator: LinearEvaluator,
}

impl EnemyDistance {
    pub fn build() -> EnemyDistanceBuilder {
        EnemyDistanceBuilder::default()
    }
}

pub fn enemy_distance(
    enemy_q: Query<&Position, With<EvilDorf>>,
    hero_q: Query<&Position, With<Hero>>,
    mut scorer_q: Query<(&Actor, &mut Score, &EnemyDistance)>,
) {
    for (Actor(actor), mut score, enemy_distance) in scorer_q.iter_mut() {
        if let Ok(enemy_pos) = enemy_q.get(*actor) {
            if let Ok(hero_pos) = hero_q.single() {
                let distance = util::euclidean_distance(enemy_pos, hero_pos);
                if distance <= enemy_distance.within {
                    score.set(enemy_distance.evaluator.evaluate(distance / enemy_distance.within));
                } else {
                    score.set(0.0);
                }
            }
        }
        if let Ok(hero_pos) = hero_q.get(*actor) {
            for enemy_pos in enemy_q.iter() {
                let distance = util::euclidean_distance(enemy_pos, hero_pos);
                if distance <= enemy_distance.within {
                    score.set(enemy_distance.evaluator.evaluate(distance / enemy_distance.within));
                } else {
                    score.set(0.0);
                }
            }
        }
    }
}
