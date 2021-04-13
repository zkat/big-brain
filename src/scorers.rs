use std::sync::Arc;

use bevy::prelude::*;

use crate::thinker::{Actor, ScorerEnt};

#[derive(Debug, Clone, Default)]
pub struct Score(pub(crate) f32);

impl Score {
    pub fn get(&self) -> f32 {
        self.0
    }
    pub fn set(&mut self, value: f32) {
        if !(0.0..=100.0).contains(&value) {
            panic!("Score value must be between 0.0 and 100.0");
        }
        self.0 = value;
    }
}

pub trait ScorerBuilder: std::fmt::Debug + Sync + Send {
    fn build(&self, cmd: &mut Commands, scorer: Entity, actor: Entity);
    fn attach(&self, cmd: &mut Commands, actor: Entity) -> Entity {
        let scorer_ent = cmd.spawn().id();
        cmd.entity(scorer_ent)
            .insert(Score::default())
            .insert(Actor(actor));
        self.build(cmd, scorer_ent, actor);
        scorer_ent
    }
}

#[derive(Debug, Clone)]
pub struct FixedScore(f32);

impl FixedScore {
    pub fn build(score: f32) -> FixedScoreBuilder {
        FixedScoreBuilder(score)
    }
}

pub fn fixed_score_system(mut query: Query<(&FixedScore, &mut Score)>) {
    for (FixedScore(fixed), mut score) in query.iter_mut() {
        score.set(*fixed);
    }
}

#[derive(Debug, Clone)]
pub struct FixedScoreBuilder(f32);

impl ScorerBuilder for FixedScoreBuilder {
    fn build(&self, cmd: &mut Commands, action: Entity, _actor: Entity) {
        cmd.entity(action).insert(FixedScore(self.0));
    }
}

#[derive(Debug)]
pub struct AllOrNothing {
    threshold: f32,
    scorers: Vec<ScorerEnt>,
}

impl AllOrNothing {
    pub fn build(threshold: f32) -> AllOrNothingBuilder {
        AllOrNothingBuilder {
            threshold,
            scorers: Vec::new(),
        }
    }
}

pub fn all_or_nothing_system(query: Query<(Entity, &AllOrNothing)>, mut scores: Query<&mut Score>) {
    for (
        aon_ent,
        AllOrNothing {
            threshold,
            scorers: children,
        },
    ) in query.iter()
    {
        let mut sum = 0.0;
        for ScorerEnt(child) in children.iter() {
            let score = scores.get_mut(*child).expect("where is it?");
            if score.0 < *threshold {
                sum = 0.0;
                break;
            } else {
                sum += score.0;
            }
        }
        let mut score = scores.get_mut(aon_ent).expect("where did it go?");
        score.set(crate::evaluators::clamp(sum, 0.0, 100.0));
    }
}
#[derive(Debug, Clone)]
pub struct AllOrNothingBuilder {
    threshold: f32,
    scorers: Vec<Arc<dyn ScorerBuilder>>,
}

impl AllOrNothingBuilder {
    pub fn when(&mut self, scorer: impl ScorerBuilder + 'static) -> &mut Self {
        self.scorers.push(Arc::new(scorer));
        self
    }
}

impl ScorerBuilder for AllOrNothingBuilder {
    fn build(&self, cmd: &mut Commands, scorer: Entity, actor: Entity) {
        let scorers: Vec<_> = self
            .scorers
            .iter()
            .map(|scorer| scorer.attach(cmd, actor))
            .collect();
        cmd.entity(scorer)
            .insert(Score::default())
            .push_children(&scorers[..])
            .insert(AllOrNothing {
                threshold: self.threshold,
                scorers: scorers.into_iter().map(ScorerEnt).collect(),
            });
    }
}

#[derive(Debug)]
pub struct SumOfScorers {
    threshold: f32,
    scorers: Vec<ScorerEnt>,
}

impl SumOfScorers {
    pub fn build(threshold: f32) -> SumOfScorersBuilder {
        SumOfScorersBuilder {
            threshold,
            scorers: Vec::new(),
        }
    }
}

pub fn sum_of_scorers_system(query: Query<(Entity, &SumOfScorers)>, mut scores: Query<&mut Score>) {
    for (
        sos_ent,
        SumOfScorers {
            threshold,
            scorers: children,
        },
    ) in query.iter()
    {
        let mut sum = 0.0;
        for ScorerEnt(child) in children.iter() {
            let score = scores.get_mut(*child).expect("where is it?");
            sum += score.0;
        }
        if sum < *threshold {
            sum = 0.0;
        }
        let mut score = scores.get_mut(sos_ent).expect("where did it go?");
        score.set(crate::evaluators::clamp(sum, 0.0, 100.0));
    }
}

#[derive(Debug, Clone)]
pub struct SumOfScorersBuilder {
    threshold: f32,
    scorers: Vec<Arc<dyn ScorerBuilder>>,
}

impl SumOfScorersBuilder {
    pub fn when(&mut self, scorer: impl ScorerBuilder + 'static) -> &mut Self {
        self.scorers.push(Arc::new(scorer));
        self
    }
}

impl ScorerBuilder for SumOfScorersBuilder {
    fn build(&self, cmd: &mut Commands, scorer: Entity, actor: Entity) {
        let scorers: Vec<_> = self
            .scorers
            .iter()
            .map(|scorer| scorer.attach(cmd, actor))
            .collect();
        cmd.entity(scorer).insert(AllOrNothing {
            threshold: self.threshold,
            scorers: scorers.into_iter().map(ScorerEnt).collect(),
        });
    }
}
