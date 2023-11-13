use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};

use crate::query::QueryVar;

#[derive(Debug)]
pub enum QueryResult {
    /// select fields from object. requires [select(object)]
    Selector(String, Vec<(String, Option<QueryVar>)>),
    /// default for named-structs: select fields from object. accepts [var(...)]
    Object(Vec<(String, QueryVar)>),
    /// todo: default for tuple-structs
    Tuple(Vec<QueryVar>),
    /// requires empty struct
    Direct(QueryVar),
}

/// will be code that writes to fmt
impl ToTokens for QueryResult {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            QueryResult::Selector(fr, vals) => {
                let (names, vars) = vals
                    .iter()
                    .map(|(n, v)| match v {
                        Some(v) => (n, quote! {::std::option::Option::Some(#v.to_string())}),
                        None => (n, quote! {::std::option::Option::None::<String>}),
                    })
                    .unzip::<_, _, Vec<_>, Vec<_>>();

                tokens.append_all(quote! {
                    fmt.write_fmt(
                        format_args!(
                            "select {} {{\n\t{}\n}}",
                            #fr,
                            [#( (#names, #vars) ),*].map(
                                |(n, v)| match v {
                                    Some(v) => format!("{} := ({})", n, v),
                                    None => String::from(n)
                            }).join::<&str>(",\n\t")
                        )
                    )?;
                })
            }
            QueryResult::Object(mapping) => {
                let mapping_tuples = mapping.iter().map(|(k, v)| {
                    quote! {
                        (#k, #v)
                    }
                });

                tokens.append_all(quote! {
                    fmt.write_fmt(format_args!(
                        "select {{\n{}\n}}",
                        [#(#mapping_tuples),*]
                            .iter()
                            .map(|(k, v)| format!("\t{k} := ({v}),"))
                            .join("\n")
                    ))?;
                });
            }
            QueryResult::Tuple(vars) => {
                tokens.append_all(quote! {
                    fmt.write_fmt(format_args!(
                        "select ({})",
                        [#( #vars ),*]
                            .iter()
                            .map(|v| format!("({})", v))
                            .join(", ")
                    ))?;
                });
            }
            QueryResult::Direct(direct) => {
                tokens.append_all(quote! {
                    fmt.write_str(
                        #direct
                    )?;
                });
            }
        };
    }
}
