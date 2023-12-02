use std::collections::HashMap;

use darling::{ast, error::Accumulator, util, Error};
use itertools::Itertools;

use strum_macros::{EnumDiscriminants, EnumTryAs};
use syn::{
    parse::Parse, punctuated::Punctuated, spanned::Spanned, token::Comma, Attribute, Expr, FnArg,
    LitStr, MetaList, Pat, Type,
};

use crate::{
    opts::ComposableQueryReturn,
    query::{Params, Query, QueryVar, With},
    selector::QuerySelector,
};
use strum_macros::IntoStaticStr;

#[derive(Debug, EnumTryAs, Clone)]
pub enum ComposableQueryAttribute {
    Params(Params),
    With(With),
    // todo: #[select(something)] -> selector
    Select(QueryVar),
    // todo: #[direct(something)]
    Direct(String),
}

impl ComposableQueryAttribute {
    fn by_discr<'a, T>(
        attrs: &'a Vec<Self>,
        unwrapper: impl Fn(&'a Self) -> Option<T> + 'a,
    ) -> impl Iterator<Item = T> + 'a {
        attrs.into_iter().filter_map(unwrapper)
    }

    fn by_discr_at_most_one<'a, T>(
        errors: &'a mut Accumulator,
        attrs: &'a Vec<Self>,
        unwrapper: impl Fn(&'a Self) -> Option<T> + 'a,
        name: &'static str,
    ) -> Option<T> {
        let res = Self::by_discr(attrs, unwrapper).at_most_one();

        match res {
            Ok(Some(a)) => Some(a),
            Ok(None) => None,
            Err(e) => {
                errors.push(Error::custom(format!(
                    "expected at most one #[{name}] attribute",
                )));
                None
            }
        }
    }

    pub fn into_query(
        attrs: Vec<Self>,
        fields: &ast::Data<util::Ignored, ComposableQueryReturn>,
    ) -> darling::Result<Query> {
        let mut errors = darling::Error::accumulator();

        let params = Self::by_discr_at_most_one(
            &mut errors,
            &attrs,
            ComposableQueryAttribute::try_as_params_ref,
            "params",
        )
        .cloned()
        .unwrap_or_default();

        let mut withs = Self::by_discr(&attrs, ComposableQueryAttribute::try_as_with_ref)
            .cloned()
            .collect::<Vec<_>>();

        let selector = Self::by_discr_at_most_one(
            &mut errors,
            &attrs,
            ComposableQueryAttribute::try_as_select_ref,
            "select",
        )
        .cloned();

        let direct = Self::by_discr_at_most_one(
            &mut errors,
            &attrs,
            ComposableQueryAttribute::try_as_direct_ref,
            "direct",
        )
        .cloned();

        if direct.is_some() && selector.is_some() {
            errors.push(Error::custom(
                "expected at most one of #[select] or #[direct]",
            ));
        }

        let result: Result<QuerySelector, &str> = (|| {
            let ast::Data::Struct(fields) = fields else {
                return Err("enums are not supported");
            };

            if fields.is_empty() {
                match direct {
                    Some(name) => {
                        return Ok(QuerySelector::Direct(
                            QueryVar::Var(name.clone()),
                            syn::parse2::<Type>(quote::quote! { () }).unwrap(),
                        ))
                    }
                    _ => {
                        return Err("expected #[direct] attribute for empty structs");
                    }
                }
            }

            if fields.fields[0].field_name.is_none() {
                if fields.fields.len() != 1 {
                    return Err("expected a single unnamed field (todo: tuples?)");
                }

                /*
                (select something) (innertype_selector)
                */

                let selector = direct
                    .map(|d| QueryVar::Var(d))
                    .or(selector)
                    .ok_or_else(|| {
                        "expected #[select] or #[direct] attribute for wrapper structs"
                    })?;

                let name = "_selector".to_string();

                withs.push(With(name.clone(), selector.clone()));

                return Ok(QuerySelector::Direct(
                    QueryVar::Var(name),
                    fields.fields[0].ty.clone(),
                ));
            }

            if direct.is_some() {
                return Err("expected no #[direct] attribute for non-empty structs");
            }

            if let Some(selector_from) = selector {
                let vars_to_select = fields
                    .iter()
                    .map(|f| {
                        (
                            f.field_name
                                .clone()
                                .expect("We thought we have named fields here"),
                            f.into(),
                        )
                    })
                    .collect_vec();

                if let Some(s) = selector_from.as_simple_name_or_ref() {
                    return Ok((QuerySelector::Selector(s.to_owned(), vars_to_select)));
                }

                let name = "_selector".to_string();

                withs.push(With(name.clone(), selector_from.clone()));

                return Ok((QuerySelector::Selector(name, vars_to_select)));
            }

            Ok((QuerySelector::Object(
                fields
                    .iter()
                    .map(|f| {
                        let fname = f
                            .field_name
                            .as_ref()
                            .cloned()
                            .expect("We thought we have named fields here");
                        (fname.clone(), f.var.clone().unwrap_or(QueryVar::Var(fname)))
                    })
                    .collect_vec(),
            )))
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
            result,
            params,
            withs,
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

    fn parse_selector(kind: &str, item: &MetaList) -> darling::Result<Self> {
        match kind {
            "select" => Ok(Self::Select(item.parse_args::<QueryVar>()?)),
            "direct" => Ok(Self::Direct(item.parse_args::<LitStr>()?.value())),
            _ => unreachable!(),
        }
    }

    fn from_meta(item: &syn::Meta) -> darling::Result<Option<Self>> {
        let item = item.require_list()?;
        let ident = item.path.require_ident()?.to_string();

        match &*ident {
            "params" => Self::parse_params(item).map(Some),
            "with" => Self::parse_with(item).map(Some),
            "select" | "direct" => Self::parse_selector(&ident, item).map(Some),
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
