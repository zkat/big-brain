use bevy::prelude::*;
use big_brain::{
    evaluators::{Evaluator, PowerEvaluator},
    prelude::*,
};

use crate::components::Hp;

pub struct FearOfDeath {
    evaluator: PowerEvaluator,
}

impl FearOfDeath {
    pub fn build() -> FearOfDeathBuilder {
        FearOfDeathBuilder::default()
    }
}
#[derive(Debug, Default)]
pub struct FearOfDeathBuilder;

impl ScorerBuilder for FearOfDeathBuilder {
    fn build(&self, cmd: &mut Commands, scorer: Entity, _actor: Entity) {
        cmd.entity(scorer).insert(FearOfDeath {
            evaluator: PowerEvaluator::new_ranged(2., 1.0, 0.0),
        });
    }
}

pub fn fear_of_death(hp_q: Query<&Hp>, mut scorer_q: Query<(&Actor, &mut Score, &FearOfDeath)>) {
    for (Actor(actor), mut score, fear) in scorer_q.iter_mut() {
        if let Ok(hp) = hp_q.get(*actor) {
            let perc = hp.current as f32 / hp.max as f32;
            score.set(fear.evaluator.evaluate(perc));
        }
    }
}
