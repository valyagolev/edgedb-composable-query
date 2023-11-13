use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::Type;

use crate::{fields::ComposableQueryReturn, query::QueryVar};

#[derive(Debug)]
pub enum SelectorValue {
    SubSelector(Type),
    Computed(QueryVar),
}

impl From<&ComposableQueryReturn> for SelectorValue {
    fn from(cqr: &ComposableQueryReturn) -> Self {
        match &cqr.var {
            Some(v) => SelectorValue::Computed(v.clone()),
            None => SelectorValue::SubSelector(cqr.ty.clone()),
        }
    }
}

#[derive(Debug)]
pub enum QueryResult {
    /// select fields from object. requires [select(object)]
    /// as query return: `select obj {field, field2: ...}`
    /// as subquery return: `select outerobj {thisfield {field, field2: ...}}`
    Selector(String, Vec<(String, SelectorValue)>),

    /// default for named-structs: select fields from object. accepts [var(...)]
    /// as query return: `select {field := a, field2 := b}
    /// as subquery return: `select outerobj {thisfield := {field := a, field2 := b}}`
    Object(Vec<(String, QueryVar)>),
    /// todo: default for tuple-structs
    /// ?
    Tuple(Vec<QueryVar>),
    /// requires empty struct
    /// as query return: `whatever`
    /// as subquery return:  `select outerobj {thisfield := (whatever)}`
    Direct(QueryVar),
}

impl QueryResult {
    pub fn as_composable_query_result_type(&self) -> TokenStream {
        match self {
            QueryResult::Selector(..) => {
                quote! {::edgedb_composable_query::ComposableQueryResultType::Selector}
            }
            QueryResult::Object(..) => {
                quote! {::edgedb_composable_query::ComposableQueryResultType::Selector}
            }
            _ => {
                quote! {::edgedb_composable_query::ComposableQueryResultType::Field}
            }
        }
    }
}

/// will be code that writes to fmt
impl ToTokens for QueryResult {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            QueryResult::Selector(fr, vals) => {
                let (names, vars) = vals
                    .iter()
                    .map(|(n, v)| match v {
                        SelectorValue::SubSelector(ty) => {
                            let ty = ty;
                            dbg!(ty);
                            (
                                n,
                                quote! {{
                                    let mut buf = String::new();
                                    <#ty as ::edgedb_composable_query::ComposableQuerySelector>::format_subquery(&mut buf)?;

                                    ::edgedb_composable_query::query_add_indent(&buf)
                                }},
                            )
                        }
                        SelectorValue::Computed(v) => {
                            (n, quote! {format!(":= ({})", #v.to_string())})
                        }
                    })
                    .unzip::<_, _, Vec<_>, Vec<_>>();

                tokens.append_all(quote! {
                    fmt.write_fmt(
                        format_args!(
                            "{{\n\t{}\n}}",
                            [#( (#names, #vars) ),*].map(
                                |(n, v)| format!("{}{}", n, v)).join::<&str>(",\n\t")
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
                        "{{\n{}\n}}",
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
                        "({})",
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
