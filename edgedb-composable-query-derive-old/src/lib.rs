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

#[cfg(test)]
fn derive_composable_query_for_test(
    item: proc_macro2::TokenStream,
) -> darling::Result<proc_macro2::TokenStream> {
    let item = syn::parse2::<DeriveInput>(item)?;

    derive_composable_query_impl(item)
}

#[proc_macro_derive(ComposableQuery, attributes(params, with, var, select, direct))]
pub fn derive_composable_query(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = syn::parse_macro_input!(item as DeriveInput);

    match derive_composable_query_impl(item) {
        Ok(ts) => ts.into(),
        Err(e) => e.write_errors().into(),
    }
}

#[proc_macro_derive(EdgedbComposableSelector, attributes(params, with, var))]
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

            #[derive(EdgedbComposableSelector)]
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
