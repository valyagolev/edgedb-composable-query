use darling::ast::{self};

use darling::{util, FromDeriveInput, FromField};

use fields::ComposableQueryReturn;
use query::QueryVar;
use quote::quote;

use syn::{DeriveInput, Type};
use tokens::ComposableQueryAttribute;

mod fields;
mod query;
mod selector;
mod tokens;

#[derive(Debug, FromDeriveInput)]
#[darling(forward_attrs(allow, doc, cfg, params, with, var, select, direct))]
struct ComposableQueryOpts {
    ident: syn::Ident,
    attrs: Vec<syn::Attribute>,
    data: ast::Data<util::Ignored, ComposableQueryReturn>,
}

fn derive_composable_query_impl(item: DeriveInput) -> darling::Result<proc_macro2::TokenStream> {
    let selector_impl = derive_composable_query_selector_impl(item.clone(), false)?;

    let item = ComposableQueryOpts::from_derive_input(&item)?;
    let attribs = ComposableQueryAttribute::from_attrs(&item.attrs)?;
    let query = ComposableQueryAttribute::into_query(attribs, &item.data)?;
    let selector = &query.result;
    let ident = &item.ident;

    Ok(quote! {
        #selector_impl

        impl ::edgedb_composable_query::ComposableQuery for #ident {
            #query
        }
    })
}

fn derive_composable_query_selector_impl(
    item: DeriveInput,
    selector_only: bool,
) -> darling::Result<proc_macro2::TokenStream> {
    let item = ComposableQueryOpts::from_derive_input(&item)?;
    let mut attribs = ComposableQueryAttribute::from_attrs(&item.attrs)?;

    if selector_only {
        attribs.push(ComposableQueryAttribute::Select(QueryVar::Var(
            "".to_string(),
        )));
    }

    let query = ComposableQueryAttribute::into_query(attribs, &item.data)?;
    let selector = &query.result;
    let ident = &item.ident;

    let result_type = selector.as_composable_query_result_type();

    Ok(quote! {
        impl ::edgedb_composable_query::AsEdgedbVar for #ident {
            const EDGEDB_TYPE_NAME: Option<&'static str> = None;
            const IS_OPTIONAL: bool = false;

            fn as_query_arg(&self) -> ::edgedb_protocol::value::Value {
                // (*self).into()
                // dbg!(self);
                todo!("1");
            }

            fn from_query_result(t: ::edgedb_protocol::value::Value) -> Self {
                dbg!(&t);
                todo!("2");
            }
        }

        impl ::edgedb_composable_query::ComposableQuerySelector for #ident {
            const RESULT_TYPE: ::edgedb_composable_query::ComposableQueryResultKind =
                #result_type;

            fn format_selector(fmt: &mut impl ::std::fmt::Write) -> Result<(), std::fmt::Error> {
                use ::edgedb_composable_query::itertools::Itertools;

                #selector

                Ok(())
            }
        }
    })
}

#[cfg(test)]
fn derive_composable_query_for_test(
    item: proc_macro2::TokenStream,
) -> darling::Result<proc_macro2::TokenStream> {
    let item = syn::parse2::<DeriveInput>(item)?;

    derive_composable_query_impl(item)
}

#[cfg(test)]
fn derive_composable_query_selector_for_test(
    item: proc_macro2::TokenStream,
) -> darling::Result<proc_macro2::TokenStream> {
    let item = syn::parse2::<DeriveInput>(item)?;

    derive_composable_query_selector_impl(item, true)
}

#[proc_macro_derive(ComposableQuery, attributes(params, with, var, select, direct))]
pub fn derive_composable_query(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = syn::parse_macro_input!(item as DeriveInput);

    match derive_composable_query_impl(item) {
        Ok(ts) => ts.into(),
        Err(e) => e.write_errors().into(),
    }
}

#[proc_macro_derive(ComposableQuerySelector, attributes(params, with, var))]
pub fn derive_composable_query_selector(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = syn::parse_macro_input!(item as DeriveInput);

    match derive_composable_query_selector_impl(item, true) {
        Ok(ts) => ts.into(),
        Err(e) => e.write_errors().into(),
    }
}

#[cfg(test)]
mod test {
    use crate::{derive_composable_query_for_test, derive_composable_query_selector_for_test};
    use proc_macro2::TokenStream;
    use quote::quote;

    fn on_one_quote(input: TokenStream) -> String {
        let out = derive_composable_query_for_test(input).unwrap();

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

    fn on_one_quote_selector(input: TokenStream) -> String {
        let out = derive_composable_query_selector_for_test(input).unwrap();

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
    fn insta_test_struct() {
        let input = quote! {

            #[derive(ComposableQuery)]
            #[params(n: i32, v: String)]
            #[with(q = crate::InsertQ2(n = "a + 1", v = "v"))]
            // #[with(calc = "n + 2")]
            // #[with(q = "insert Q {n := calc, name := n}")]
            // #[with(calc2 = calc)]
            #[select("select Inner limit 1")]
            struct InsertQ {
                // this is for `select { q := q, calc := calc }`
                // #[var(q)]
                id: String,
                #[var("calc")]
                calc: i32,
                by_name: i32,
            }

        };

        let formatted = on_one_quote(input);

        insta::assert_snapshot!(formatted);
    }

    #[test]
    fn insta_test_struct2() {
        let input = quote! {

            #[derive(ComposableQuery)]
            struct Inner {
                id: Uuid,
                opt: Option<String>,
                req: String,

                #[var("len(.req)")]
                strlen: i64,
            }

        };

        let formatted = on_one_quote(input);

        insta::assert_snapshot!(formatted);
    }

    #[test]
    fn insta_test_struct_wrapper() {
        let input = quote! {

            #[derive(ComposableQuery)]
            #[params(id: Uuid)]
            #[select("select Inner filter .id = id limit 1")]
            struct InnerById(Inner);

        };

        let formatted = on_one_quote(input);

        insta::assert_snapshot!(formatted);
    }

    #[test]
    fn insta_test_struct_selector() {
        let input = quote! {

            #[derive(ComposableQuerySelector)]
            struct Inner {
                id: Uuid,
                opt: Option<String>,
                req: String,

                #[var("len(.req)")]
                strlen: i64,
            }

        };

        let formatted = on_one_quote_selector(input);

        insta::assert_snapshot!(formatted);
    }

    #[test]
    fn insta_test_empty_struct() {
        let input = quote! {

            #[derive(ComposableQuery)]
            #[params(n: i32)]
            #[direct("select User limit 1")]
            struct ReshuffleTuple;

        };

        let formatted = on_one_quote(input);

        insta::assert_snapshot!(formatted);
    }

    #[test]
    fn insta_test_tuple_named() {
        let input = quote! {

            #[derive(ComposableQuery)]
            #[params(n: i32, v: String)]
            struct ReshuffleTuple(
                #[var("v")]
                String,
                #[var("n")]
                i32,
            );

        };

        let formatted = on_one_quote(input);

        insta::assert_snapshot!(formatted);
    }

    #[test]
    fn insta_test_tuple_direct() {
        let input = quote! {

            #[derive(ComposableQuery)]
            #[params(n: i32, v: String)]
            #[direct(v, n)]
            struct ReshuffleTuple(i32, String,);

        };

        let formatted = on_one_quote(input);

        insta::assert_snapshot!(formatted);
    }
}
