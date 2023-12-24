use bevy::prelude::*;
use big_brain::prelude::*;

#[derive(Debug, Clone, Component, ActionBuilder)]
#[action_label = "MyLabel"]
pub struct MyAction;

#[test]
fn check_macro() {
    let action = MyAction;
    assert_eq!(action.label(), Some("MyLabel"))
}

#[derive(Debug, Clone, Component, ActionBuilder)]
#[action_label = "MyGenericLabel"]
pub struct MyGenericAction<T: Clone + Send + Sync + std::fmt::Debug + 'static> {
    pub value: T,
}

#[test]
fn check_generic_macro() {
    let action = MyGenericAction { value: 0 };
    assert_eq!(action.label(), Some("MyGenericLabel"))
}

#[derive(Debug, Clone, Component, ActionBuilder)]
#[action_label = "MyGenericWhereLabel"]
pub struct MyGenericWhereAction<T>
where
    T: Clone + Send + Sync + std::fmt::Debug + 'static,
{
    pub value: T,
}

#[test]
fn check_generic_where_macro() {
    let action = MyGenericWhereAction { value: 0 };
    assert_eq!(action.label(), Some("MyGenericWhereLabel"))
}
