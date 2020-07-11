use darling::{ast, FromDeriveInput, FromField, ToTokens};
use proc_macro2::TokenStream;
use quote::quote;
use syn;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(action), supports(struct_named))]
pub struct Action {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<(), ActionField>,
}

#[derive(Debug, FromField)]
#[darling(attributes(action))]
struct ActionField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    #[darling(default)]
    param: bool,
    #[darling(default)]
    default: bool,
}

impl ToTokens for Action {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Action {
            ref ident,
            ref data,
            ..
        } = *self;
        let fields = data
            .as_ref()
            .take_struct()
            .expect("Enums not supported")
            .fields;
        let field_defs = fields.clone().into_iter().filter_map(|field| {
            let ActionField {
                ident, ty, param, ..
            } = field;
            let ident = ident.clone().unwrap();
            if *param && ident != syn::Ident::new("actor", ident.span()) {
                Some(quote! { #ident: #ty })
            } else {
                None
            }
        });
        let field_assignments = fields.into_iter().map(|field| {
            let ActionField {
                ident,
                param,
                default,
                ..
            } = field;
            let ident = ident.clone().unwrap();
            if *param {
                quote! {
                    #ident: self.#ident
                }
            } else if *default {
                quote! {
                    #ident: ::core::default::Default::default()
                }
            } else if ident == syn::Ident::new("actor", ident.span()) {
                quote! {
                    #ident: actor
                }
            } else {
                panic!("Field not state, default, or param: {}", ident);
            }
        });
        let ts = quote! {
            mod big_brain_builder {
                use super::#ident as Comp;

                use big_brain::{typetag, serde::Deserialize, Action, ActionManager, ecs::{Entity, Entities, LazyUpdate}, ActionEnt};

                #[derive(Debug, Deserialize)]
                struct #ident {
                    #(#field_defs),*
                }

                #[typetag::deserialize]
                impl Action for #ident {
                    fn build(self: Box<Self>, actor: Entity, action_ent: ActionEnt, ents: &Entities, lazy: &LazyUpdate) -> Box<dyn ActionManager> {
                        self
                    }
                }

                impl ActionManager for #ident {
                    fn activate(&self, actor: Entity, action_ent: ActionEnt, lazy: &LazyUpdate) {
                        lazy.insert(action_ent.0.clone(), Comp {
                            #(#field_assignments),*
                        });
                    }
                    fn deactivate(&self, action_ent: ActionEnt, lazy: &LazyUpdate) {
                        lazy.remove::<Comp>(action_ent.0);
                    }
                }
            }
        };
        tokens.extend(ts);
    }
}
