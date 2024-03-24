//! Derive ScorerBuilder on a given struct
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident, Lit, LitStr, Meta};

/// Derive ScorerBuilder on a struct that implements Component + Clone
pub fn scorer_builder_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let label = get_label(&input);

    let component_name = input.ident;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let component_string = component_name.to_string();
    let build_method = build_method(&component_name, &ty_generics);
    let label_method = label_method(
        label.unwrap_or_else(|| LitStr::new(&component_string, component_name.span())),
    );

    let gen = quote! {
        impl #impl_generics ::big_brain::scorers::ScorerBuilder for #component_name #ty_generics #where_clause {
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
                if ident == "scorer_label" {
                    if let Lit::Str(lit_str) = lit {
                        label = Some(lit_str);
                    } else {
                        panic!("Must specify a string for the `scorer_label` attribute")
                    }
                }
            }
        }
    }
    label
}

fn build_method(component_name: &Ident, ty_generics: &syn::TypeGenerics) -> TokenStream {
    let turbofish = ty_generics.as_turbofish();

    quote! {
        fn build(&self, cmd: &mut ::bevy::ecs::system::Commands, scorer: ::bevy::ecs::entity::Entity, _actor: ::bevy::ecs::entity::Entity) {
            cmd.entity(scorer).insert(#component_name  #turbofish::clone(self));
        }
    }
}

fn label_method(label: LitStr) -> TokenStream {
    quote! {
        fn label(&self) -> ::std::option::Option<&str> {
            ::std::option::Option::Some(#label)
        }
    }
}
