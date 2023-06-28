//! Scorers look at the world and boil down arbitrary characteristics into a
//! range of 0.0..=1.0. This module includes the ScorerBuilder trait and some
//! built-in Composite Scorers.

use std::{cmp::Ordering, sync::Arc};

use bevy::prelude::*;
#[cfg(feature = "trace")]
use bevy::utils::tracing::trace;

use crate::{
    evaluators::Evaluator,
    measures::{Measure, WeightedMeasure},
    thinker::{Actor, Scorer, ScorerSpan},
};

/// Score value between `0.0..=1.0` associated with a Scorer.
#[derive(Clone, Component, Debug, Default, Reflect)]
pub struct Score(pub(crate) f32);

impl Score {
    /// Returns the `Score`'s current value.
    pub fn get(&self) -> f32 {
        self.0
    }

    /// Set the `Score`'s value.
    ///
    /// ### Panics
    ///
    /// Panics if `value` isn't within `0.0..=1.0`.
    pub fn set(&mut self, value: f32) {
        if !(0.0..=1.0).contains(&value) {
            panic!("Score value must be between 0.0 and 1.0");
        }
        self.0 = value;
    }

    /// Set the `Score`'s value. Allows values outside the range `0.0..=1.0`
    /// Note: it is up to you to make sure your scorers are using the same units!
    pub fn set_unchecked(&mut self, value: f32) {
        self.0 = value;
    }
}

/// Trait that must be defined by types in order to be `ScorerBuilder`s.
/// `ScorerBuilder`s' job is to spawn new `Scorer` entities. In general, most
/// of this is already done for you, and the only method you really have to
/// implement is `.build()`.
///
/// The `build()` method MUST be implemented for any `ScorerBuilder`s you want
/// to define.
#[reflect_trait]
pub trait ScorerBuilder: std::fmt::Debug + Sync + Send {
    /// MUST insert your concrete Scorer component into the Scorer [`Entity`],
    ///  using `cmd`. You _may_ use `actor`, but it's perfectly normal to just
    ///  ignore it.
    ///
    /// In most cases, your `ScorerBuilder` and `Scorer` can be the same type.
    /// The only requirement is that your struct implements `Debug`,
    /// `Component, `Clone`. You can then use the derive macro `ScorerBuilder`
    /// to turn your struct into a `ScorerBuilder`
    ///
    /// ### Example
    ///
    /// Using the derive macro (the easy way):
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use big_brain::prelude::*;
    /// #[derive(Debug, Clone, Component, ScorerBuilder)]
    /// #[scorer_label = "MyScorerLabel"] // Optional. Defaults to type name.
    /// struct MyScorer;
    /// ```
    ///
    /// Implementing it manually:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use big_brain::prelude::*;
    /// #[derive(Debug)]
    /// struct MyBuilder;
    /// #[derive(Debug, Component)]
    /// struct MyScorer;
    ///
    /// impl ScorerBuilder for MyBuilder {
    ///   fn build(&self, cmd: &mut Commands, scorer: Entity, _actor: Entity) {
    ///     cmd.entity(scorer).insert(MyScorer);
    ///   }
    /// }
    /// ```
    fn build(&self, cmd: &mut Commands, scorer: Entity, actor: Entity);

    /// A label to display when logging using the Scorer's tracing span.
    fn label(&self) -> Option<&str> {
        None
    }
}

pub fn spawn_scorer<T: ScorerBuilder + ?Sized>(
    builder: &T,
    cmd: &mut Commands,
    actor: Entity,
) -> Entity {
    let scorer_ent = cmd.spawn_empty().id();
    let span = ScorerSpan::new(scorer_ent, ScorerBuilder::label(builder));
    let _guard = span.span().enter();
    debug!("New Scorer spawned.");
    cmd.entity(scorer_ent)
        .insert(Name::new("Scorer"))
        .insert(Score::default())
        .insert(Actor(actor));
    builder.build(cmd, scorer_ent, actor);
    std::mem::drop(_guard);
    cmd.entity(scorer_ent).insert(span);
    scorer_ent
}

/// Scorer that always returns the same, fixed score. Good for combining with
/// things creatively!
#[derive(Clone, Component, Debug, Reflect)]
pub struct FixedScore(pub f32);

impl FixedScore {
    pub fn build(score: f32) -> FixedScorerBuilder {
        FixedScorerBuilder { score, label: None }
    }
}

pub fn fixed_score_system(mut query: Query<(&FixedScore, &mut Score, &ScorerSpan)>) {
    for (FixedScore(fixed), mut score, _span) in query.iter_mut() {
        #[cfg(feature = "trace")]
        {
            let _guard = _span.span().enter();
            trace!("FixedScore: {}", fixed);
        }
        score.set(*fixed);
    }
}

#[derive(Debug, Reflect)]
pub struct FixedScorerBuilder {
    score: f32,
    label: Option<String>,
}

impl FixedScorerBuilder {
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl ScorerBuilder for FixedScorerBuilder {
    fn build(&self, cmd: &mut Commands, scorer: Entity, _actor: Entity) {
        cmd.entity(scorer).insert(FixedScore(self.score));
    }

    fn label(&self) -> Option<&str> {
        self.label.as_deref().or(Some("FixedScore"))
    }
}

/// Composite Scorer that takes any number of other Scorers and returns the
/// sum of their [`Score`] values if each _individual_ [`Score`] is at or
/// above the configured `threshold`.
///
/// ### Example
///
/// ```
/// # use bevy::prelude::*;
/// # use big_brain::prelude::*;
/// # #[derive(Debug, Clone, Component, ScorerBuilder)]
/// # struct MyScorer;
/// # #[derive(Debug, Clone, Component, ScorerBuilder)]
/// # struct MyOtherScorer;
/// # #[derive(Debug, Clone, Component, ActionBuilder)]
/// # struct MyAction;
/// # fn main() {
/// Thinker::build()
///     .when(
///         AllOrNothing::build(0.8)
///           .push(MyScorer)
///           .push(MyOtherScorer),
///         MyAction);
/// # ;
/// # }
/// ```
#[derive(Component, Debug, Reflect)]
pub struct AllOrNothing {
    threshold: f32,
    scorers: Vec<Scorer>,
}

impl AllOrNothing {
    pub fn build(threshold: f32) -> AllOrNothingBuilder {
        AllOrNothingBuilder {
            threshold,
            scorers: Vec::new(),
            scorer_labels: Vec::new(),
            label: None,
        }
    }
}

pub fn all_or_nothing_system(
    query: Query<(Entity, &AllOrNothing, &ScorerSpan)>,
    mut scores: Query<&mut Score>,
) {
    for (
        aon_ent,
        AllOrNothing {
            threshold,
            scorers: children,
        },
        _span,
    ) in query.iter()
    {
        let mut sum = 0.0;
        for Scorer(child) in children.iter() {
            let score = scores.get_mut(*child).expect("where is it?");
            if score.0 < *threshold {
                sum = 0.0;
                break;
            } else {
                sum += score.0;
            }
        }
        let mut score = scores.get_mut(aon_ent).expect("where did it go?");
        score.set(crate::evaluators::clamp(sum, 0.0, 1.0));
        #[cfg(feature = "trace")]
        {
            let _guard = _span.span().enter();
            trace!("AllOrNothing score: {}", score.get());
        }
    }
}

#[derive(Debug, Clone, Reflect)]
pub struct AllOrNothingBuilder {
    threshold: f32,
    #[reflect(ignore)]
    scorers: Vec<Arc<dyn ScorerBuilder>>,
    scorer_labels: Vec<String>,
    label: Option<String>,
}

impl AllOrNothingBuilder {
    /// Add another Scorer to this [`ScorerBuilder`].
    pub fn push(mut self, scorer: impl ScorerBuilder + 'static) -> Self {
        if let Some(label) = scorer.label() {
            self.scorer_labels.push(label.into());
        } else {
            self.scorer_labels.push("Unnamed Scorer".into());
        }
        self.scorers.push(Arc::new(scorer));
        self
    }

    /// Set a label for this Action.
    pub fn label(mut self, label: impl AsRef<str>) -> Self {
        self.label = Some(label.as_ref().into());
        self
    }
}

impl ScorerBuilder for AllOrNothingBuilder {
    fn label(&self) -> Option<&str> {
        self.label.as_deref().or(Some("AllOrNothing"))
    }

    fn build(&self, cmd: &mut Commands, scorer: Entity, actor: Entity) {
        let scorers: Vec<_> = self
            .scorers
            .iter()
            .map(|scorer| spawn_scorer(&**scorer, cmd, actor))
            .collect();
        cmd.entity(scorer)
            .insert(Score::default())
            .push_children(&scorers[..])
            .insert(Name::new("Scorer"))
            .insert(AllOrNothing {
                threshold: self.threshold,
                scorers: scorers.into_iter().map(Scorer).collect(),
            });
    }
}

/// Composite Scorer that takes any number of other Scorers and returns the sum of their [`Score`] values if the _total_ summed [`Score`] is at or above the configured `threshold`.
///
/// ### Example
///
/// ```
/// # use bevy::prelude::*;
/// # use big_brain::prelude::*;
/// # #[derive(Debug, Clone, Component, ScorerBuilder)]
/// # struct MyScorer;
/// # #[derive(Debug, Clone, Component, ScorerBuilder)]
/// # struct MyOtherScorer;
/// # #[derive(Debug, Clone, Component, ActionBuilder)]
/// # struct MyAction;
/// # fn main() {
/// Thinker::build()
///     .when(
///         SumOfScorers::build(0.8)
///           .push(MyScorer)
///           .push(MyOtherScorer),
///         MyAction)
/// # ;
/// # }
/// ```
#[derive(Component, Debug, Reflect)]
pub struct SumOfScorers {
    threshold: f32,
    scorers: Vec<Scorer>,
    scorer_labels: Vec<String>,
}

impl SumOfScorers {
    pub fn build(threshold: f32) -> SumOfScorersBuilder {
        SumOfScorersBuilder {
            threshold,
            scorers: Vec::new(),
            scorer_labels: Vec::new(),
            label: None,
        }
    }
}

pub fn sum_of_scorers_system(
    query: Query<(Entity, &SumOfScorers, &ScorerSpan)>,
    mut scores: Query<&mut Score>,
) {
    for (
        sos_ent,
        SumOfScorers {
            threshold,
            scorers: children,
            ..
        },
        _span,
    ) in query.iter()
    {
        let mut sum = 0.0;
        for Scorer(child) in children.iter() {
            let score = scores.get_mut(*child).expect("where is it?");
            sum += score.0;
        }
        if sum < *threshold {
            sum = 0.0;
        }
        let mut score = scores.get_mut(sos_ent).expect("where did it go?");
        score.set(crate::evaluators::clamp(sum, 0.0, 1.0));
        #[cfg(feature = "trace")]
        {
            let _guard = _span.span().enter();
            trace!(
                "SumOfScorers score: {}, from {} scores",
                score.get(),
                children.len()
            );
        }
    }
}

#[derive(Debug, Clone, Reflect)]
pub struct SumOfScorersBuilder {
    threshold: f32,
    #[reflect(ignore)]
    scorers: Vec<Arc<dyn ScorerBuilder>>,
    scorer_labels: Vec<String>,
    label: Option<String>,
}

impl SumOfScorersBuilder {
    /// Add a new Scorer to this [`SumOfScorersBuilder`].
    pub fn push(mut self, scorer: impl ScorerBuilder + 'static) -> Self {
        if let Some(label) = scorer.label() {
            self.scorer_labels.push(label.into());
        } else {
            self.scorer_labels.push("Unnamed Scorer".into());
        }
        self.scorers.push(Arc::new(scorer));
        self
    }

    /// Set a label for this Action.
    pub fn label(mut self, label: impl AsRef<str>) -> Self {
        self.label = Some(label.as_ref().into());
        self
    }
}

impl ScorerBuilder for SumOfScorersBuilder {
    fn label(&self) -> Option<&str> {
        self.label.as_deref().or(Some("SumOfScorers"))
    }

    #[allow(clippy::needless_collect)]
    fn build(&self, cmd: &mut Commands, scorer: Entity, actor: Entity) {
        let scorers: Vec<_> = self
            .scorers
            .iter()
            .map(|scorer| spawn_scorer(&**scorer, cmd, actor))
            .collect();
        cmd.entity(scorer)
            .push_children(&scorers[..])
            .insert(SumOfScorers {
                threshold: self.threshold,
                scorers: scorers.into_iter().map(Scorer).collect(),
                scorer_labels: self.scorer_labels.clone(),
            });
    }
}

/// Composite Scorer that takes any number of other Scorers and returns the
/// product of their [`Score`]. If the resulting score is less than the
/// threshold, it returns 0.
///
/// The Scorer can also apply a compensation factor based on the number of
/// Scores passed to it. This can be enabled by passing `true` to the
/// `use_compensation` method on the builder.
///
/// ### Example
///
/// ```
/// # use bevy::prelude::*;
/// # use big_brain::prelude::*;
/// # #[derive(Debug, Clone, Component, ScorerBuilder)]
/// # struct MyScorer;
/// # #[derive(Debug, Clone, Component, ScorerBuilder)]
/// # struct MyOtherScorer;
/// # #[derive(Debug, Clone, Component, ActionBuilder)]
/// # struct MyAction;
/// # fn main() {
/// Thinker::build()
///     .when(
///         ProductOfScorers::build(0.5)
///           .use_compensation(true)
///           .push(MyScorer)
///           .push(MyOtherScorer),
///         MyAction)
/// # ;
/// # }
/// ```

#[derive(Component, Debug, Reflect)]
pub struct ProductOfScorers {
    threshold: f32,
    use_compensation: bool,
    scorers: Vec<Scorer>,
    scorer_labels: Vec<String>,
}

impl ProductOfScorers {
    pub fn build(threshold: f32) -> ProductOfScorersBuilder {
        ProductOfScorersBuilder {
            threshold,
            use_compensation: false,
            scorers: Vec::new(),
            scorer_labels: Vec::new(),
            label: None,
        }
    }
}

pub fn product_of_scorers_system(
    query: Query<(Entity, &ProductOfScorers, &ScorerSpan)>,
    mut scores: Query<&mut Score>,
) {
    for (
        sos_ent,
        ProductOfScorers {
            threshold,
            use_compensation,
            scorers: children,
            ..
        },
        _span,
    ) in query.iter()
    {
        let mut product = 1.0;
        let mut num_scorers = 0;

        for Scorer(child) in children.iter() {
            let score = scores.get_mut(*child).expect("where is it?");
            product *= score.0;
            num_scorers += 1;
        }

        // See for example
        // http://www.gdcvault.com/play/1021848/Building-a-Better-Centaur-AI
        if *use_compensation && product < 1.0 {
            let mod_factor = 1.0 - 1.0 / (num_scorers as f32);
            let makeup = (1.0 - product) * mod_factor;
            product += makeup * product;
        }

        if product < *threshold {
            product = 0.0;
        }

        let mut score = scores.get_mut(sos_ent).expect("where did it go?");
        score.set(product.clamp(0.0, 1.0));
        #[cfg(feature = "trace")]
        {
            let _guard = _span.span().enter();
            trace!(
                "ProductOfScorers score: {}, from {} scores",
                score.get(),
                children.len()
            );
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProductOfScorersBuilder {
    threshold: f32,
    use_compensation: bool,
    scorers: Vec<Arc<dyn ScorerBuilder>>,
    scorer_labels: Vec<String>,
    label: Option<String>,
}

impl ProductOfScorersBuilder {
    /// To account for the fact that the total score will be reduced for
    /// scores with more inputs, we can optionally apply a compensation factor
    /// by calling this and passing `true`
    pub fn use_compensation(mut self, use_compensation: bool) -> Self {
        self.use_compensation = use_compensation;
        self
    }

    /// Add a new scorer to this [`ProductOfScorersBuilder`].
    pub fn push(mut self, scorer: impl ScorerBuilder + 'static) -> Self {
        if let Some(label) = scorer.label() {
            self.scorer_labels.push(label.into());
        } else {
            self.scorer_labels.push("Unnamed Scorer".into());
        }
        self.scorers.push(Arc::new(scorer));
        self
    }

    /// Set a label for this Action.
    pub fn label(mut self, label: impl AsRef<str>) -> Self {
        self.label = Some(label.as_ref().into());
        self
    }
}

impl ScorerBuilder for ProductOfScorersBuilder {
    fn label(&self) -> Option<&str> {
        self.label.as_deref().or(Some("ProductOfScorers"))
    }

    #[allow(clippy::needless_collect)]
    fn build(&self, cmd: &mut Commands, scorer: Entity, actor: Entity) {
        let scorers: Vec<_> = self
            .scorers
            .iter()
            .map(|scorer| spawn_scorer(&**scorer, cmd, actor))
            .collect();
        cmd.entity(scorer)
            .push_children(&scorers[..])
            .insert(ProductOfScorers {
                threshold: self.threshold,
                use_compensation: self.use_compensation,
                scorers: scorers.into_iter().map(Scorer).collect(),
                scorer_labels: self.scorer_labels.clone(),
            });
    }
}

/// Composite Scorer that takes any number of other Scorers and returns the
/// single highest value [`Score`] if  _any_ [`Score`]s are at or above the
/// configured `threshold`.
///
/// ### Example
///
/// ```
/// # use bevy::prelude::*;
/// # use big_brain::prelude::*;
/// # #[derive(Debug, Clone, Component, ScorerBuilder)]
/// # struct MyScorer;
/// # #[derive(Debug, Clone, Component, ScorerBuilder)]
/// # struct MyOtherScorer;
/// # #[derive(Debug, Clone, Component, ActionBuilder)]
/// # struct MyAction;
/// # fn main() {
/// Thinker::build()
///     .when(
///         WinningScorer::build(0.8)
///           .push(MyScorer)
///           .push(MyOtherScorer),
///         MyAction)
/// # ;
/// # }
/// ```

#[derive(Component, Debug, Reflect)]
pub struct WinningScorer {
    threshold: f32,
    scorers: Vec<Scorer>,
    scorer_labels: Vec<String>,
}

impl WinningScorer {
    pub fn build(threshold: f32) -> WinningScorerBuilder {
        WinningScorerBuilder {
            threshold,
            scorers: Vec::new(),
            scorer_labels: Vec::new(),
            label: None,
        }
    }
}

pub fn winning_scorer_system(
    mut query: Query<(Entity, &mut WinningScorer, &ScorerSpan)>,
    mut scores: Query<&mut Score>,
) {
    for (sos_ent, mut winning_scorer, _span) in query.iter_mut() {
        let (threshold, children) = (winning_scorer.threshold, &mut winning_scorer.scorers);
        let mut all_scores = children
            .iter()
            .map(|Scorer(e)| scores.get(*e).expect("where is it?"))
            .collect::<Vec<&Score>>();

        all_scores.sort_by(|a, b| a.get().partial_cmp(&b.get()).unwrap_or(Ordering::Equal));
        let winning_score_or_zero = match all_scores.last() {
            Some(s) => {
                if s.get() < threshold {
                    0.0
                } else {
                    s.get()
                }
            }
            None => 0.0,
        };
        let mut score = scores.get_mut(sos_ent).expect("where did it go?");
        score.set(crate::evaluators::clamp(winning_score_or_zero, 0.0, 1.0));
        #[cfg(feature = "trace")]
        {
            let _guard = _span.span().enter();
            trace!(
                "WinningScorer score: {}, from {} scores",
                score.get(),
                children.len()
            );
        }
    }
}

#[derive(Debug, Clone, Reflect)]
pub struct WinningScorerBuilder {
    threshold: f32,
    #[reflect(ignore)]
    scorers: Vec<Arc<dyn ScorerBuilder>>,
    scorer_labels: Vec<String>,
    label: Option<String>,
}

impl WinningScorerBuilder {
    /// Add another Scorer to this [`WinningScorerBuilder`].
    pub fn push(mut self, scorer: impl ScorerBuilder + 'static) -> Self {
        if let Some(label) = scorer.label() {
            self.scorer_labels.push(label.into());
        } else {
            self.scorer_labels.push("Unnamed Scorer".into())
        }
        self.scorers.push(Arc::new(scorer));
        self
    }

    /// Set a label for this Action.
    pub fn label(mut self, label: impl AsRef<str>) -> Self {
        self.label = Some(label.as_ref().into());
        self
    }
}

impl ScorerBuilder for WinningScorerBuilder {
    fn label(&self) -> Option<&str> {
        self.label.as_deref().or(Some("WinningScorer"))
    }

    #[allow(clippy::needless_collect)]
    fn build(&self, cmd: &mut Commands, scorer: Entity, actor: Entity) {
        let scorers: Vec<_> = self
            .scorers
            .iter()
            .map(|scorer| spawn_scorer(&**scorer, cmd, actor))
            .collect();
        cmd.entity(scorer)
            .push_children(&scorers[..])
            .insert(WinningScorer {
                threshold: self.threshold,
                scorers: scorers.into_iter().map(Scorer).collect(),
                scorer_labels: self.scorer_labels.clone(),
            });
    }
}

/// Composite scorer that takes a `ScorerBuilder` and applies an `Evaluator`.
/// Note that unlike other composite scorers, `EvaluatingScorer` only takes
/// one scorer upon building.
///
/// ### Example
///
/// ```
/// # use bevy::prelude::*;
/// # use big_brain::prelude::*;
/// # #[derive(Debug, Clone, Component, ScorerBuilder)]
/// # struct MyScorer;
/// # #[derive(Debug, Clone, Component, ActionBuilder)]
/// # struct MyAction;
/// # #[derive(Debug, Clone)]
/// # struct MyEvaluator;
/// # impl Evaluator for MyEvaluator {
/// #    fn evaluate(&self, score: f32) -> f32 {
/// #        score
/// #    }
/// # }
/// # fn main() {
/// Thinker::build()
///     .when(
///         EvaluatingScorer::build(MyScorer, MyEvaluator),
///         MyAction)
/// # ;
/// # }
/// ```
#[derive(Component, Debug, Reflect)]
pub struct EvaluatingScorer {
    scorer: Scorer,
    evaluator_string: String,
    #[reflect(ignore)]
    evaluator: Arc<dyn Evaluator>,
}

impl EvaluatingScorer {
    pub fn build(
        scorer: impl ScorerBuilder + 'static,
        evaluator: impl Evaluator + 'static,
    ) -> EvaluatingScorerBuilder {
        EvaluatingScorerBuilder {
            scorer_label: scorer.label().map(|s| s.into()),
            evaluator: Arc::new(evaluator),
            scorer: Arc::new(scorer),
            label: None,
        }
    }
}

pub fn evaluating_scorer_system(
    query: Query<(Entity, &EvaluatingScorer, &ScorerSpan)>,
    mut scores: Query<&mut Score>,
) {
    for (sos_ent, eval_scorer, _span) in query.iter() {
        // Get the inner score
        let inner_score = scores
            .get(eval_scorer.scorer.0)
            .expect("where did it go?")
            .get();
        // Get composite score
        let mut score = scores.get_mut(sos_ent).expect("where did it go?");
        score.set(crate::evaluators::clamp(
            eval_scorer.evaluator.evaluate(inner_score),
            0.0,
            1.0,
        ));
        #[cfg(feature = "trace")]
        {
            let _guard = _span.span().enter();
            trace!(
                "EvaluatingScorer score: {}, from score: {}",
                score.get(),
                inner_score
            );
        }
    }
}

#[derive(Debug, Reflect)]
pub struct EvaluatingScorerBuilder {
    #[reflect(ignore)]
    scorer: Arc<dyn ScorerBuilder>,
    scorer_label: Option<String>,
    #[reflect(ignore)]
    evaluator: Arc<dyn Evaluator>,
    label: Option<String>,
}

impl ScorerBuilder for EvaluatingScorerBuilder {
    fn label(&self) -> Option<&str> {
        self.label.as_deref().or(Some("EvaluatingScorer"))
    }

    fn build(&self, cmd: &mut Commands, scorer: Entity, actor: Entity) {
        let inner_scorer = spawn_scorer(&*self.scorer, cmd, actor);
        let scorers = vec![inner_scorer];
        cmd.entity(scorer)
            .push_children(&scorers[..])
            .insert(EvaluatingScorer {
                evaluator: self.evaluator.clone(),
                scorer: Scorer(inner_scorer),
                evaluator_string: format!("{:#?}", self.evaluator),
            });
    }
}

/// Composite Scorer that allows more fine-grained control of how the scores
/// are combined. The default is to apply a weighting
///
/// ### Example
///
/// Using the default measure ([`WeightedMeasure`]):
///
/// ```
/// # use bevy::prelude::*;
/// # use big_brain::prelude::*;
/// # #[derive(Debug, Clone, Component, ScorerBuilder)]
/// # struct MyScorer;
/// # #[derive(Debug, Clone, Component, ScorerBuilder)]
/// # struct MyOtherScorer;
/// # #[derive(Debug, Clone, Component, ActionBuilder)]
/// # struct MyAction;
/// # fn main() {
/// Thinker::build()
///     .when(
///         MeasuredScorer::build(0.5)
///           .push(MyScorer, 0.9)
///           .push(MyOtherScorer, 0.4),
///         MyAction)
/// # ;
/// # }
/// ```
///
/// Customising the measure:
///
/// ```
/// # use bevy::prelude::*;
/// # use big_brain::prelude::*;
/// # #[derive(Debug, Clone, Component, ScorerBuilder)]
/// # struct MyScorer;
/// # #[derive(Debug, Clone, Component, ScorerBuilder)]
/// # struct MyOtherScorer;
/// # #[derive(Debug, Clone, Component, ActionBuilder)]
/// # struct MyAction;
/// # fn main() {
/// Thinker::build()
///     .when(
///         MeasuredScorer::build(0.5)
///           .measure(ChebyshevDistance)
///           .push(MyScorer, 0.8)
///           .push(MyOtherScorer, 0.2),
///         MyAction)
/// # ;
/// # }
/// ```

#[derive(Component, Debug, Reflect)]
pub struct MeasuredScorer {
    threshold: f32,
    #[reflect(ignore)]
    measure: Arc<dyn Measure>,
    measure_string: String,
    scorers: Vec<(Scorer, f32)>,
}

impl MeasuredScorer {
    pub fn build(threshold: f32) -> MeasuredScorerBuilder {
        MeasuredScorerBuilder {
            threshold,
            measure: Arc::new(WeightedMeasure),
            measure_string: format!("{WeightedMeasure:#?}"),
            scorers: Vec::new(),
            scorer_labels: Vec::new(),
            label: None,
        }
    }
}

pub fn measured_scorers_system(
    query: Query<(Entity, &MeasuredScorer, &ScorerSpan)>,
    mut scores: Query<&mut Score>,
) {
    for (
        sos_ent,
        MeasuredScorer {
            threshold,
            measure,
            scorers: children,
            ..
        },
        _span,
    ) in query.iter()
    {
        let measured_score = measure.calculate(
            children
                .iter()
                .map(|(scorer, weight)| (scores.get(scorer.0).expect("where is it?"), *weight))
                .collect::<Vec<_>>(),
        );
        let mut score = scores.get_mut(sos_ent).expect("where did it go?");

        if measured_score < *threshold {
            score.set(0.0);
        } else {
            score.set(measured_score.clamp(0.0, 1.0));
        }
        #[cfg(feature = "trace")]
        {
            let _guard = _span.span().enter();
            trace!(
                "MeasuredScorer score: {}, from {} scores",
                score.get(),
                children.len()
            );
        }
    }
}

#[derive(Debug, Reflect)]
pub struct MeasuredScorerBuilder {
    threshold: f32,
    #[reflect(ignore)]
    measure: Arc<dyn Measure>,
    measure_string: String,
    #[reflect(ignore)]
    scorers: Vec<(Arc<dyn ScorerBuilder>, f32)>,
    scorer_labels: Vec<String>,
    label: Option<String>,
}

impl MeasuredScorerBuilder {
    /// Sets the measure to be used to combine the child scorers
    pub fn measure(mut self, measure: impl Measure + 'static) -> Self {
        self.measure_string = format!("{measure:#?}");
        self.measure = Arc::new(measure);
        self
    }

    pub fn push(mut self, scorer: impl ScorerBuilder + 'static, weight: f32) -> Self {
        if let Some(label) = scorer.label() {
            self.scorer_labels.push(label.into());
        } else {
            self.scorer_labels.push("Unnamed Scorer".into());
        }
        self.scorers.push((Arc::new(scorer), weight));
        self
    }

    /// Set a label for this ScorerBuilder.
    pub fn label(mut self, label: impl AsRef<str>) -> Self {
        self.label = Some(label.as_ref().into());
        self
    }
}

impl ScorerBuilder for MeasuredScorerBuilder {
    fn label(&self) -> Option<&str> {
        self.label.as_deref().or(Some("MeasuredScorer"))
    }

    #[allow(clippy::needless_collect)]
    fn build(&self, cmd: &mut Commands, scorer: Entity, actor: Entity) {
        let scorers: Vec<_> = self
            .scorers
            .iter()
            .map(|(scorer, _)| spawn_scorer(&**scorer, cmd, actor))
            .collect();
        cmd.entity(scorer)
            .push_children(&scorers[..])
            .insert(MeasuredScorer {
                threshold: self.threshold,
                measure: self.measure.clone(),
                scorers: scorers
                    .into_iter()
                    .map(Scorer)
                    .zip(self.scorers.iter().map(|(_, weight)| *weight))
                    .collect(),
                measure_string: self.measure_string.clone(),
            });
    }
}
