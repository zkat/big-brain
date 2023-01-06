//! Big Brain Derive
//! Procedural macros to simplify the implementation of Big Brain traits
mod action;
mod scorer;

use action::action_builder_impl;
use scorer::scorer_builder_impl;

/// Derives ActionBuilder for a struct that implements Component + Clone
#[proc_macro_derive(ActionBuilder, attributes(label))]
pub fn action_builder_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    action_builder_impl(input)
}

/// Derives ScorerBuilder for a struct that implements Component + Clone
#[proc_macro_derive(ScorerBuilder, attributes(label))]
pub fn scorer_builder_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    scorer_builder_impl(input)
}
