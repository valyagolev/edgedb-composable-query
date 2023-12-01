use object::derive_edgedb_object_impl;
use syn::DeriveInput;

mod object;

// #[proc_macro_derive(EdgedbObject, attributes(params, with, var, select, direct))]
#[proc_macro_derive(EdgedbObject)]
pub fn derive_edgedb_object(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = syn::parse_macro_input!(item as DeriveInput);

    match derive_edgedb_object_impl(item) {
        Ok(ts) => ts.into(),
        Err(e) => e.write_errors().into(),
    }
}
