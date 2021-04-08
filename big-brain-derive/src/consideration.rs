use darling::{ast, FromDeriveInput, FromField, ToTokens};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(consideration), supports(struct_named))]
pub struct Consideration {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<(), ConsiderationField>,
}

#[derive(Debug, FromField)]
#[darling(attributes(consideration))]
struct ConsiderationField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    #[darling(default)]
    param: bool,
    #[darling(default)]
    default: bool,
}

impl ToTokens for Consideration {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Consideration {
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
            let ConsiderationField {
                ident, ty, param, ..
            } = field;
            let ident = ident.clone().unwrap();
            if *param && ident != syn::Ident::new("parent", ident.span()) {
                Some(quote! { #ident: #ty })
            } else {
                None
            }
        });
        let field_assignments = fields.into_iter().map(|field| {
            let ConsiderationField {
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
            mod big_brain_cons_builder {
                use super::#ident as Comp;

                use big_brain::{typetag, serde::Deserialize, Consideration, bevy::prelude::*, ConsiderationEnt};
                // use typetag;

                #[derive(Debug, Deserialize)]
                struct #ident {
                    #(#field_defs),*
                }
                #[typetag::deserialize]
                impl Consideration for #ident {
                    fn build(&self, actor: Entity, cmd: &mut Commands) -> ConsiderationEnt {
                        let ent = ConsiderationEnt(cmd.spawn().id());
                        cmd.entity(ent.0)
                        .insert(big_brain::Utility::default())
                        .insert(Comp {
                            #(#field_assignments),*
                        });
                        cmd.entity(actor).push_children(&[ent.0]);
                        ent
                    }
                }
            }
        };
        tokens.extend(ts);
    }
}
