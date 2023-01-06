use bevy::prelude::*;
use big_brain::prelude::*;

#[derive(Debug, Clone, Component, ScorerBuilder)]
#[label = "MyLabel"]
pub struct MyScorer;

#[test]
fn check_macro() {
    // TODO: how to make sure that the struct implements Component and Clone?

    let scorer = MyScorer;
    assert_eq!(scorer.label(), Some("MyLabel"))
}
