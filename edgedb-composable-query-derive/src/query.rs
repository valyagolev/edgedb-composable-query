use std::collections::HashMap;

use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use quote::{ToTokens, TokenStreamExt};
use syn::{Path, Type};

use crate::selector::QuerySelector;

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

// #[derive(Debug)]
// pub enum QueryResult {
//     QuerySelector(QuerySelector),
//     InnerType(Type),
// }

#[derive(Debug)]
pub struct Query {
    pub params: Params,
    pub withs: Vec<With>,
    pub result: QuerySelector,
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

/// will be code that writes to fmt
impl ToTokens for Params {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for (name, ty) in self.0.iter() {
            tokens.append_all(quote! {
                fmt.write_fmt(format_args!(
                    "\t{} := <{}>{},\n",
                    #name,
                    <#ty as ::edgedb_composable_query::EdgedbPrim>::TYPE_CAST,
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

                        <#strct as ::edgedb_composable_query::EdgedbComposableQuery>::format_query(&mut buf, &args)?;

                        ::edgedb_composable_query::__query_add_indent(&buf)
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

        // self.result.to_tokens(&mut inner);

        let (argnames, argtypes) = self
            .params
            .0
            .iter()
            .cloned()
            .unzip::<_, _, Vec<String>, Vec<Type>>();

        let self_type = quote! {Self};

        let (final_selector, final_type) = match &self.result {
            QuerySelector::Selector(what, _) => (quote! {format!("select ({})", #what)}, self_type),
            QuerySelector::Object(_) => (quote! {"select "}, self_type),
            // QuerySelector::Tuple(_) => quote! {"select "},
            QuerySelector::Direct(what, _ty) => {
                (quote! {format!("select ({})", #what)}, quote! {#_ty})
            }
        };

        let atypes = if argtypes.len() == 0 {
            quote! {()}
        } else {
            quote! {(#( #argtypes ),* ,)}
        };

        tokens.append_all(quote! {
            const ARG_NAMES: &'static [&'static str] = &[#( #argnames ),*];

            type ArgTypes = #atypes;
            type ReturnType = #final_type;

            fn format_query(
                fmt: &mut impl ::std::fmt::Write,
                args: &::std::collections::HashMap<&str, String>
            ) -> Result<(), ::std::fmt::Error> {
                use ::edgedb_composable_query::__itertools::Itertools;
                use ::edgedb_composable_query::composable::EdgedbComposableSelector;

                #inner

                fmt.write_str(&#final_selector)?;
                fmt.write_str(" {\n")?;

                <#final_type as EdgedbComposableSelector>::format_selector(fmt)?;

                fmt.write_str("\n}")?;

                Ok(())
            }
        })
    }
}
