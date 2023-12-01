use darling::ast::{self};

use darling::{util, FromDeriveInput, FromField};

use quote::quote;

use syn::{DeriveInput, Type};

#[derive(Debug, FromField)]
// #[darling(attributes(lorem))]
pub struct EdgedbObjectField {
    ident: Option<syn::Ident>,
    ty: Type,
    // #[darling(default)]
    // skip: bool,
}

#[derive(Debug, FromDeriveInput)]
#[darling(forward_attrs(allow, doc, cfg))]
struct EdgedbObjectOpts {
    ident: syn::Ident,
    attrs: Vec<syn::Attribute>,
    data: ast::Data<util::Ignored, EdgedbObjectField>,
}

pub fn derive_edgedb_object_impl(item: DeriveInput) -> darling::Result<proc_macro2::TokenStream> {
    let item = EdgedbObjectOpts::from_derive_input(&item)?;
    let fields = item.data.take_struct().ok_or_else(|| {
        darling::Error::custom("expected struct with named fields").with_span(&item.ident)
    })?;

    let field_names = fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect::<Vec<_>>();

    Ok(quote! {

        fn from_edgedb_object(
            shape: edgedb_protocol::codec::ObjectShape,
            mut fields: Vec<Option<edgedb_protocol::value::Value>>,
        ) -> anyhow::Result<Self> {
            #(
                let mut #field_names = None;
            )*;

            for (i, s) in shape.elements.iter().enumerate() {
                match s.name.as_str() {
                    #(
                        stringify!(#field_names) => {
                            #field_names = fields[i]
                                .take()
                                .map(EdgedbSetValue::from_edgedb_set_value)
                                .transpose()?;
                        }
                    )*
                }
            }

            Ok(Self {
                #(
                    #field_names: EdgedbSetValue::interpret_possibly_missing_required_value(#field_names)?,
                )*
            })
        }

        // fn to_edgedb_object(
        //     &self,
        // ) -> anyhow::Result<(
        //     edgedb_protocol::codec::ObjectShape,
        //     Vec<Option<edgedb_protocol::value::Value>>,
        // )> {
        //     todo!()
        // }
    })
}

#[cfg(test)]
fn derive_edgedb_object_for_test(
    item: proc_macro2::TokenStream,
) -> darling::Result<proc_macro2::TokenStream> {
    let item = syn::parse2::<DeriveInput>(item)?;

    derive_edgedb_object_impl(item)
}
