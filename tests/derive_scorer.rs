use bevy::prelude::*;
use big_brain::prelude::*;

#[derive(Debug, Clone, Component, ScorerBuilder)]
#[scorer_label = "MyLabel"]
pub struct MyScorer;

#[test]
fn check_macro() {
    // TODO: how to make sure that the struct implements Component and Clone?

    let scorer = MyScorer;
    assert_eq!(scorer.label(), Some("MyLabel"))
}

#[derive(Debug, Clone, Component, ScorerBuilder)]
#[scorer_label = "MyGenericLabel"]
pub struct MyGenericScorer<T: Clone + Send + Sync + std::fmt::Debug + 'static> {
    pub value: T,
}

#[test]
fn check_generic_macro() {
    let scorer = MyGenericScorer { value: 0 };
    assert_eq!(scorer.label(), Some("MyGenericLabel"))
}

#[derive(Debug, Clone, Component, ScorerBuilder)]
#[scorer_label = "MyGenericWhereLabel"]
pub struct MyGenericWhereScorer<T>
    where
        T: Clone + Send + Sync + std::fmt::Debug + 'static,
{
    pub value: T,
}

#[test]
fn check_generic_where_macro() {
    let scorer = MyGenericWhereScorer { value: 0 };
    assert_eq!(scorer.label(), Some("MyGenericWhereLabel"))
}
