use std::collections::HashMap;

use darling::{ast, util, Error};
use itertools::Itertools;

use syn::{
    parse::Parse, punctuated::Punctuated, spanned::Spanned, token::Comma, Attribute, Expr, FnArg,
    MetaList, Pat,
};

use crate::{
    query::{Params, Query, QueryResult, QueryVar, With},
    ComposableQueryReturn,
};

#[derive(Debug)]
pub enum ComposableQueryAttribute {
    Params(Params),
    With(With),
    // todo: #[select(something)] -> selector
    Select(QueryVar),
    // todo: #[direct(something)]
    Direct(String),
}

impl ComposableQueryAttribute {
    pub fn into_query(
        attrs: Vec<Self>,
        fields: &ast::Data<util::Ignored, ComposableQueryReturn>,
    ) -> darling::Result<Query> {
        let mut errors = darling::Error::accumulator();

        let params = match attrs
            .iter()
            .filter(|p| matches!(p, ComposableQueryAttribute::Params(_)))
            .exactly_one()
        {
            Ok(ComposableQueryAttribute::Params(params)) => params.clone(),
            _ => {
                errors.push(Error::custom("expected exactly one #[params] attribute"));

                Default::default()
            }
        };

        let mut withs = attrs
            .iter()
            .filter_map(|p| match p {
                ComposableQueryAttribute::With(w) => Some(w),
                _ => None,
            })
            .cloned()
            .collect::<Vec<_>>();

        let selector = attrs
            .iter()
            .filter(|p| matches!(p, ComposableQueryAttribute::Select(_)))
            .at_most_one();

        if selector.is_err() {
            errors.push(Error::custom("expected at most one #[select] attribute"));
        }

        let selector = selector.unwrap_or_default();

        let direct = attrs
            .iter()
            .filter(|p| matches!(p, ComposableQueryAttribute::Direct(_)))
            .at_most_one();

        if direct.is_err() {
            errors.push(Error::custom("expected at most one #[direct] attribute"));
        }

        let direct = direct.unwrap_or_default();

        if direct.is_some() && selector.is_some() {
            errors.push(Error::custom(
                "expected at most one of #[select] or #[direct]",
            ));
        }

        let result: Result<QueryResult, &str> = (|| {
            let ast::Data::Struct(fields) = fields else {
                return Err("enums are not supported");
            };

            if fields.is_empty() {
                match direct {
                    Some(ComposableQueryAttribute::Direct(name)) => {
                        return Ok(QueryResult::Direct(QueryVar::Var(name.clone())))
                    }
                    _ => {
                        return Err("expected #[direct] attribute for empty structs");
                    }
                }
            }

            if fields.fields[0].ident.is_none() {
                todo!("tuple structs");
            }

            if direct.is_some() {
                return Err("expected no #[direct] attribute for non-empty structs");
            }

            if let Some(ComposableQueryAttribute::Select(selector_from)) = selector {
                let vars_to_select = fields
                    .iter()
                    .map(|f| QueryVar::Var(f.ident.as_ref().unwrap().to_string()))
                    .collect_vec();

                if let Some(s) = selector_from.as_simple_name_or_ref() {
                    return Ok(QueryResult::Selector(s.to_owned(), vars_to_select));
                }

                let name = "_selector".to_string();

                withs.push(With(name.clone(), selector_from.clone()));

                return Ok(QueryResult::Selector(name, vars_to_select));
            }

            Ok(QueryResult::Object(
                fields
                    .iter()
                    .map(|f| (f.ident.as_ref().unwrap().to_string(), f.var.clone()))
                    .collect_vec(),
            ))
        })();

        let result = match result {
            Ok(r) => r,
            Err(e) => {
                errors.push(Error::custom(e));
                errors.finish()?;

                unreachable!()
            }
        };

        errors.finish()?;

        Ok(Query {
            params,
            withs,
            result,
        })
    }

    pub fn from_attrs(attrs: &[Attribute]) -> darling::Result<Vec<Self>> {
        let mut errors = darling::Error::accumulator();

        let res = attrs
            .iter()
            .filter_map(|a| {
                errors
                    .handle_in(|| ComposableQueryAttribute::from_meta(&a.meta))
                    .flatten()
            })
            .collect::<Vec<ComposableQueryAttribute>>();

        errors.finish_with(res)
    }

    fn parse_params(item: &MetaList) -> darling::Result<Self> {
        let args = item.parse_args_with(Punctuated::<FnArg, Comma>::parse_terminated)?;

        let args: Vec<_> = args
            .iter()
            .map(|p| match p {
                FnArg::Receiver(_) => Err(darling::Error::unexpected_type(
                    "can't have a receiver param",
                )),
                FnArg::Typed(at) => {
                    let Pat::Ident(name) = at.pat.as_ref() else {
                        return Err(darling::Error::unexpected_type(
                            "expected a simple identifier",
                        ));
                    };
                    let name = name.ident.to_string();

                    Ok((name, *at.ty.clone()))
                }
            })
            .map_ok(|p| p.clone())
            .collect::<darling::Result<_>>()?;

        Ok(Self::Params(Params(args)))
    }

    fn parse_with(item: &MetaList) -> darling::Result<Self> {
        let mut with = None;

        // dbg!(item);

        item.parse_nested_meta(|arg| {
            let name = arg.path.require_ident()?.to_string();
            let template = arg.value()?;

            with = Some(With(name, template.parse::<QueryVar>()?));

            Ok(())
        })?;

        let with =
            with.ok_or_else(|| darling::Error::custom("expected args for with").with_span(&item))?;

        Ok(Self::With(with))
    }

    fn from_meta(item: &syn::Meta) -> darling::Result<Option<Self>> {
        let item = item.require_list()?;
        let ident = item.path.require_ident()?.to_string();

        match &*ident {
            "params" => Self::parse_params(item).map(Some),
            "with" => Self::parse_with(item).map(Some),
            _ => Ok(None),
        }
    }
}

impl QueryVar {
    fn from_expr(expr: syn::Expr) -> syn::Result<Self> {
        let span = expr.span();
        match expr {
            syn::Expr::Lit(syn::ExprLit { lit, .. }) => {
                let s = match lit {
                    syn::Lit::Str(s) => s,
                    _ => {
                        return Err(syn::Error::new(
                            span,
                            "expected a string literal or a struct name",
                        ))
                    }
                };

                let template = s.value();

                Ok(QueryVar::Var(template))
            }
            syn::Expr::Path(path) => {
                let name = path.path.require_ident()?.to_string();

                Ok(QueryVar::Var(name))
            }
            syn::Expr::Call(call) => {
                let Expr::Path(strct) = *call.func else {
                    return Err(syn::Error::new(span, "expected a struct name"));
                };

                let strct = strct.path;

                let bindings = call
                    .args
                    .iter()
                    .map(|exp| {
                        if let Expr::Assign(assign) = exp {
                            let Expr::Path(name) = *assign.left.clone() else {
                                return Err(syn::Error::new(
                                    exp.span(),
                                    "expected a simple named argument (e.g. name = value)",
                                ));
                            };

                            let name = name.path.require_ident()?.to_string();

                            Ok((name, Self::from_expr(*assign.right.clone())?))
                        } else {
                            Err(syn::Error::new(
                                exp.span(),
                                "expected a named argument (e.g. name = value)",
                            ))
                        }
                    })
                    .collect::<Result<HashMap<_, _>, _>>()?;

                Ok(QueryVar::Call(strct, bindings))
            }
            _ => Err(syn::Error::new(span, "expected a struct name")),
        }
    }
}

impl Parse for QueryVar {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // dbg!(&input);

        if let Ok(expr) = input.parse::<syn::Expr>() {
            // dbg!(&expr);
            Self::from_expr(expr)
        } else {
            Err(syn::Error::new(
                input.span(),
                "expected a template string or a var name or a call to another query",
            ))
        }
    }
}
