use crate::{opts::ComposableQueryOpts, query::QueryVar, tokens::ComposableQueryAttribute};
use darling::FromDeriveInput;
use quote::quote;
use syn::DeriveInput;

pub fn derive_composable_selector_impl(
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
        impl ::edgedb_composable_query::composable::EdgedbComposableSelector for #ident {
            const RESULT_TYPE: ::edgedb_composable_query::composable::ComposableQueryResultKind =
                #result_type;

            fn format_selector(fmt: &mut impl ::std::fmt::Write) -> Result<(), std::fmt::Error> {
                use ::edgedb_composable_query::__itertools::Itertools;

                #selector

                Ok(())
            }
        }
    })
}

#[cfg(test)]
fn derive_composable_selector_for_test(
    item: proc_macro2::TokenStream,
) -> darling::Result<proc_macro2::TokenStream> {
    use syn::DeriveInput;

    let item = syn::parse2::<DeriveInput>(item)?;

    derive_composable_selector_impl(item, true)
}
