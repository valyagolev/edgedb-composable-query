use composable_selector::derive_composable_selector_impl;
use object::derive_edgedb_object_impl;
use syn::DeriveInput;

mod composable_query;
mod composable_selector;
mod object;
mod opts;
mod query;
mod selector;
mod tokens;

#[proc_macro_derive(EdgedbObject)]
pub fn derive_edgedb_object(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = syn::parse_macro_input!(item as DeriveInput);

    match derive_edgedb_object_impl(item) {
        Ok(ts) => ts.into(),
        Err(e) => e.write_errors().into(),
    }
}

#[proc_macro_derive(EdgedbComposableSelector, attributes(params, with, var))]
pub fn derive_edgedb_composable_selector(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = syn::parse_macro_input!(item as DeriveInput);

    match derive_composable_selector_impl(item, true) {
        Ok(ts) => ts.into(),
        Err(e) => e.write_errors().into(),
    }
}
