use bevy::prelude::*;
use big_brain::prelude::*;

#[derive(Debug, Clone, Component, ActionBuilder)]
#[label = "MyLabel"]
pub struct MyAction;

#[test]
fn check_macro() {
    let action = MyAction;
    assert_eq!(action.label(), Some("MyLabel"))
}
