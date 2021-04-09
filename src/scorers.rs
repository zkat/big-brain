use bevy::prelude::*;

use crate::ScorerEnt;

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

/**
This trait defines new Scorers. In general, you should use the [derive macro](derive.Scorer.html) instead.
*/
#[typetag::deserialize]
pub trait Scorer: std::fmt::Debug + Sync + Send {
    fn build(&self, entity: Entity, cmd: &mut Commands) -> ScorerEnt;
}

#[derive(Debug)]
pub struct FixedScore(f32);

pub fn fixed_score_system(mut query: Query<(&FixedScore, &mut Score)>) {
    for (FixedScore(fixed), mut score) in query.iter_mut() {
        score.set(*fixed);
    }
}

mod fixed_score {
    use super::*;

    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct FixedScore(f32);

    #[typetag::deserialize]
    impl Scorer for FixedScore {
        fn build(&self, actor: Entity, cmd: &mut Commands) -> ScorerEnt {
            let ent = ScorerEnt(cmd.spawn().id());
            cmd.entity(ent.0)
                .insert(Score::default())
                .insert(super::FixedScore(self.0));
            cmd.entity(actor).push_children(&[ent.0]);
            ent
        }
    }
}

#[derive(Debug)]
pub struct AllOrNothing {
    threshold: f32,
    scorers: Vec<ScorerEnt>,
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

mod all_or_nothing {
    use super::*;

    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct AllOrNothing {
        threshold: f32,
        scorers: Vec<Box<dyn Scorer>>,
    }

    #[typetag::deserialize]
    impl Scorer for AllOrNothing {
        fn build(&self, actor: Entity, cmd: &mut Commands) -> ScorerEnt {
            let ent = ScorerEnt(cmd.spawn().id());
            let scorers: Vec<_> = self
                .scorers
                .iter()
                .map(|scorer| scorer.build(actor, cmd).0)
                .collect();
            cmd.entity(ent.0)
                .insert(Score::default())
                .insert(super::AllOrNothing {
                    threshold: self.threshold,
                    scorers: scorers.into_iter().map(ScorerEnt).collect(),
                });
            cmd.entity(actor).push_children(&[ent.0]);
            ent
        }
    }
}

#[derive(Debug)]
pub struct SumOfScorers {
    threshold: f32,
    scorers: Vec<ScorerEnt>,
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

mod sum_of_scorers {
    use super::*;

    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct SumOfScorers {
        threshold: f32,
        scorers: Vec<Box<dyn Scorer>>,
    }

    #[typetag::deserialize]
    impl Scorer for SumOfScorers {
        fn build(&self, actor: Entity, cmd: &mut Commands) -> ScorerEnt {
            let ent = ScorerEnt(cmd.spawn().id());
            let scorers: Vec<_> = self
                .scorers
                .iter()
                .map(|scorer| scorer.build(actor, cmd).0)
                .collect();
            cmd.entity(ent.0)
                .insert(Score::default())
                .insert(super::AllOrNothing {
                    threshold: self.threshold,
                    scorers: scorers.into_iter().map(ScorerEnt).collect(),
                });
            cmd.entity(actor).push_children(&[ent.0]);
            ent
        }
    }
}
