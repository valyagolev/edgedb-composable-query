use crate::{opts::ComposableQueryOpts, tokens::ComposableQueryAttribute};
use darling::FromDeriveInput;
use quote::quote;
use syn::DeriveInput;

pub fn derive_composable_query_impl(
    item: DeriveInput,
) -> darling::Result<proc_macro2::TokenStream> {
    let item = ComposableQueryOpts::from_derive_input(&item)?;
    let attribs = ComposableQueryAttribute::from_attrs(&item.attrs)?;
    let query = ComposableQueryAttribute::into_query(attribs, &item.data, false)?;
    // let selector = &query.result;
    let ident = &item.ident;

    Ok(quote! {
        impl ::edgedb_composable_query::composable::EdgedbComposableQuery for #ident {
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

#[cfg(test)]
mod test {

    use proc_macro2::TokenStream;
    use quote::quote;

    use super::derive_composable_query_for_test;

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

    #[test]
    fn insta_test_struct() {
        let input = quote! {

            #[derive(EdgedbComposableQuery)]
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

            #[derive(EdgedbComposableQuery)]
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

            #[derive(EdgedbComposableQuery)]
            #[params(id: Uuid)]
            #[select("select Inner filter .id = id limit 1")]
            struct InnerById(Inner);

        };

        let formatted = on_one_quote(input);

        insta::assert_snapshot!(formatted);
    }

    #[test]
    fn insta_test_empty_struct() {
        let input = quote! {

            #[derive(EdgedbComposableQuery)]
            #[params(n: i32)]
            #[direct("select User limit 1")]
            struct ReshuffleTuple;

        };

        let formatted = on_one_quote(input);

        insta::assert_snapshot!(formatted);
    }

    #[test]
    fn insta_simpl() {
        let input = quote! {
            #[derive(
                Debug, PartialEq, Eq, EdgedbObject, EdgedbComposableSelector, EdgedbComposableQuery,
            )]
            #[select("select Inner limit 1")]
            struct InnerSelector {
                req: String,
                opt: Option<String>,
            }

        };

        let formatted = on_one_quote(input);

        insta::assert_snapshot!(formatted);
    }

    #[test]
    fn insta_test_tuple_named() {
        let input = quote! {

            #[derive(EdgedbComposableQuery)]
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

            #[derive(EdgedbComposableQuery)]
            #[params(n: i32, v: String)]
            #[direct(v, n)]
            struct ReshuffleTuple(i32, String,);

        };

        let formatted = on_one_quote(input);

        insta::assert_snapshot!(formatted);
    }

    #[test]
    fn insta_test_wrapper() {
        let input = quote! {

            #[derive(Debug, PartialEq, Eq, EdgedbComposableSelector, EdgedbComposableQuery)]
            #[select("select Inner limit 1")]
            struct OneInnerBySelector(InnerSelector);

        };

        let formatted = on_one_quote(input);

        insta::assert_snapshot!(formatted);
    }
}
