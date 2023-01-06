//! Derive ScorerBuilder on a given struct
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident, Lit, LitStr, Meta};

/// Derive ScorerBuilder on a struct that implements Component + Clone
pub fn scorer_builder_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let label = get_label(&input);

    let component_name = input.ident;
    let build_method = build_method(&component_name);
    let label_method = label_method(label);

    let gen = quote! {
        impl ScorerBuilder for #component_name {
            #build_method
            #label_method
        }
    };

    proc_macro::TokenStream::from(gen)
}

fn get_label(input: &DeriveInput) -> Option<LitStr> {
    let mut label: Option<LitStr> = None;
    let attrs = &input.attrs;
    for option in attrs {
        let option = option.parse_meta().unwrap();
        if let Meta::NameValue(meta_name_value) = option {
            let path = meta_name_value.path;
            let lit = meta_name_value.lit;
            if let Some(ident) = path.get_ident() {
                if ident == "label" {
                    if let Lit::Str(lit_str) = lit {
                        label = Some(lit_str);
                    } else {
                        panic!("Must specify a string for the `label` attribute")
                    }
                }
            }
        }
    }
    label
}

fn build_method(component_name: &Ident) -> TokenStream {
    quote! {
        fn build(&self, cmd: &mut Commands, scorer: Entity, _actor: Entity) {
            cmd.entity(scorer).insert(#component_name::clone(self));
        }
    }
}

fn label_method(label_option: Option<LitStr>) -> TokenStream {
    let inner = if let Some(label) = label_option {
        quote! {Some(#label)}
    } else {
        quote! {None}
    };
    quote! {
        fn label(&self) -> Option<&str> {
            #inner
        }
    }
}
