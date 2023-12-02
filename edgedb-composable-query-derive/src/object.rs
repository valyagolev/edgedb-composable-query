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

    let item_name = &item.ident;

    Ok(quote! {

        impl EdgedbObject for #item_name {

            fn from_edgedb_object(
                shape: edgedb_protocol::codec::ObjectShape,
                mut fields: Vec<Option<edgedb_protocol::value::Value>>,
            ) -> anyhow::Result<Self> {
                use edgedb_composable_query::EdgedbSetValue;

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
                        _ => {}
                    }
                }

                Ok(Self {
                    #(
                        #field_names: EdgedbSetValue::interpret_possibly_missing_required_value(#field_names)?,
                    )*
                })
            }
        }
    })
}

#[cfg(test)]
fn derive_edgedb_object_for_test(
    item: proc_macro2::TokenStream,
) -> darling::Result<proc_macro2::TokenStream> {
    let item = syn::parse2::<DeriveInput>(item)?;

    derive_edgedb_object_impl(item)
}

#[cfg(test)]
mod test {
    use proc_macro2::TokenStream;
    use quote::quote;

    use super::derive_edgedb_object_for_test;

    fn on_one_quote(input: TokenStream) -> String {
        let out = derive_edgedb_object_for_test(input).unwrap();

        let s = out.to_string();
        let as_file = match syn::parse_file(&s) {
            Ok(f) => f,
            Err(e) => {
                println!("{}", s);
                panic!("failed to parse output: {}", e);
            }
        };

        prettyplease::unparse(&as_file)
    }

    #[test]
    fn insta_test_tuple_direct() {
        let input = quote! {

            #[derive(Debug, PartialEq, EdgedbObject)]
            struct ExamplImplStruct {
                a: String,
                b: Option<String>,
            }

        };

        let formatted = on_one_quote(input);

        insta::assert_snapshot!(formatted);
    }
}
