use darling::{ast, util, FromDeriveInput, FromField};
use syn::Type;

use crate::query::QueryVar;

#[derive(Debug)]
pub struct ComposableQueryReturn {
    // ident: Option<syn::Ident>,
    pub field_name: Option<String>,

    #[allow(unused)]
    pub ty: Type,

    pub var: Option<QueryVar>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(forward_attrs(allow, doc, cfg, params, with, var, select, direct))]
pub struct ComposableQueryOpts {
    pub ident: syn::Ident,
    pub attrs: Vec<syn::Attribute>,
    pub data: ast::Data<util::Ignored, ComposableQueryReturn>,
}

impl FromField for ComposableQueryReturn {
    fn from_field(field: &syn::Field) -> darling::Result<Self> {
        let ident = field.ident.clone();

        let ty = field.ty.clone();

        let mut var = None;

        field.attrs.iter().try_for_each(|a| {
            if a.path().is_ident("var") {
                if var.is_none() {
                    var = Some(a.parse_args::<QueryVar>()?);
                } else {
                    return Err(
                        darling::Error::custom("expected only one var attribute").with_span(&a)
                    );
                }
            }

            Ok(())
        })?;

        // let var = var.unwrap_or_else(|| QueryVar::Var(field.ident.clone().unwrap().to_string()));

        let field_name = field.ident.clone().map(|i| i.to_string());

        Ok(Self {
            field_name,
            // ident,
            ty,
            var,
        })
    }
}
