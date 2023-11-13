use std::collections::HashMap;

use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use quote::{ToTokens, TokenStreamExt};
use syn::Path;

#[derive(Debug, Clone, Default)]
pub struct Params(pub Vec<(String, syn::Type)>);

#[derive(Debug, Clone)]
pub struct With(pub String, pub QueryVar);

#[derive(Debug, Clone)]
pub enum QueryVar {
    Var(String),

    // SomeSubQuery(k=v, k2=v2)
    // => (with ... select ...)
    Call(Path, HashMap<String, QueryVar>),
}

impl QueryVar {
    // pub fn is_simple_name_or_ref(&self) -> bool {
    //     self.as_simple_name_or_ref().is_some()
    // }

    pub fn as_simple_name_or_ref(&self) -> Option<&str> {
        let QueryVar::Var(s) = self else {
            return None;
        };

        if !s
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
        {
            return None;
        }

        Some(s)
    }
}

#[derive(Debug)]
pub enum QueryResult {
    /// select fields from object. requires [select(object)]
    Selector(String, Vec<QueryVar>),
    /// default for named-structs: select fields from object. accepts [var(...)]
    Object(Vec<(String, QueryVar)>),
    /// todo: default for tuple-structs
    Tuple(Vec<QueryVar>),
    /// requires empty struct
    Direct(QueryVar),
}

#[derive(Debug)]
pub struct Query {
    pub params: Params,
    pub withs: Vec<With>,
    pub result: QueryResult,
}

/// will be code that writes to fmt
impl ToTokens for Params {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for (name, ty) in self.0.iter() {
            tokens.append_all(quote! {
                fmt.write_fmt(format_args!(
                    "\t{} := <{}>{},\n",
                    #name,
                    <#ty as ::edgedb_composable_query::AsEdgedbVar>::EDGEDB_TYPE,
                    args[#name]
                ))?;
            })
        }
    }
}

/// will be code that returns a string/&str
impl ToTokens for QueryVar {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            QueryVar::Var(s) => tokens.append_all(quote! {
                #s
            }),
            QueryVar::Call(strct, args) => {
                let args_kv = args
                    .iter()
                    .map(|(k, v)| {
                        quote! {
                            (#k, format!("({})", #v))
                        }
                    })
                    .collect_vec();

                tokens.append_all(quote! {
                    {
                        let args = [#( #args_kv ),*].into();
                        let mut buf = String::new();

                        <#strct as ::edgedb_composable_query::ComposableQuery>::format_query(&mut buf, &args)?;

                        ::edgedb_composable_query::query_add_indent(&buf)
                    }
                })
            }
        }
    }
}

/// will be code that writes to fmt
impl ToTokens for With {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let With(name, value) = self;

        tokens.append_all(quote! {
            fmt.write_fmt(format_args!("\t{} := ({}),\n", #name, #value))?;
        })
    }
}

/// will be code that writes to fmt
impl ToTokens for QueryResult {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            QueryResult::Selector(fr, vals) => tokens.append_all(quote! {
                fmt.write_fmt(
                    format_args!(
                        "select {} {{\n\t{}\n}}",
                        #fr,
                        [#( #vals ),*].join(", ")
                    )
                )?;
            }),
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

        // tokens.append_all(quote! {
        //     fmt.write_str(#text)?;
        // })
    }
}

/// will be a function(fmt: &mut impl Write, args: &[&str])
impl ToTokens for Query {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut inner = TokenStream::new();

        if !self.params.0.is_empty() || !self.withs.is_empty() {
            inner.append_all(quote! {
                fmt.write_str("with\n")?;
            });

            self.params.to_tokens(&mut inner);

            for with in &self.withs {
                with.to_tokens(&mut inner);
            }
        }

        self.result.to_tokens(&mut inner);

        let argnames = self.params.0.iter().map(|p| p.0.as_str()).collect_vec();

        tokens.append_all(quote! {
            const ARG_NAMES: &'static [&'static str] = &[#( #argnames ),*];

            fn format_query(
                fmt: &mut impl ::std::fmt::Write,
                args: &::std::collections::HashMap<&str, String>
            ) -> Result<(), ::std::fmt::Error> {
                use ::edgedb_composable_query::itertools::Itertools;

                #inner

                Ok(())
            }
        })
    }
}
