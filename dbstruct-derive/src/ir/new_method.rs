use proc_macro2::Span;
use syn::punctuated::Punctuated;
use syn::{parse_quote, Pat, PathArguments, Token};

use crate::model::backend::Backend;
use crate::model::{Field, Model, Wrapper};

use super::struct_def::Struct;

pub struct NewMethod {
    pub locals: Vec<syn::Local>,
    pub fields: Vec<syn::FieldValue>,
    pub vis: syn::Visibility,
    pub arg: syn::FnArg,
}

fn as_len_value(ident: syn::Ident) -> syn::FieldValue {
    let colon: syn::token::Colon = syn::Token![:](Span::call_site());
    syn::FieldValue {
        attrs: Vec::new(),
        member: syn::Member::Named(ident),
        colon_token: Some(colon),
        expr: parse_quote!(std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0))),
    }
}

fn len_expr(ty: &syn::Type, prefix: u8) -> Box<syn::Expr> {
    let next_prefix = prefix + 1;
    let expr: syn::Expr = parse_quote!(
    dbstruct::traits::data_store::Orderd::get_lt(&ds, &#next_prefix)?
        .map(|(len, _): (u8, #ty)| len)
        .unwrap_or(0)
    );
    Box::new(expr)
}

fn len_init(field: &Field) -> Option<syn::Local> {
    let ty = match &field.wrapper {
        Wrapper::Vec { ty } => ty,
        _ => return None,
    };

    let ident = syn::PathSegment {
        ident: field.ident.clone(),
        arguments: PathArguments::None,
    };
    let ident = syn::Path {
        leading_colon: None,
        segments: Punctuated::from_iter(std::iter::once(ident)),
    };
    let ident = syn::PatPath {
        attrs: Vec::new(),
        qself: None,
        path: ident,
    };
    let expr = len_expr(ty, field.key);
    let eq_token = Token![=](Span::call_site());
    Some(syn::Local {
        attrs: Vec::new(),
        let_token: Token![let](Span::call_site()),
        pat: Pat::Path(ident),
        init: Some((eq_token, expr)),
        semi_token: Token![;](Span::call_site()),
    })
}

fn sled_from_path() -> syn::Local {
    let stmt: syn::Stmt = parse_quote!(
    let ds = sled::Config::default()
        .path(path)
        .open()?
        .open_tree("DbStruct")?;
    );
    match stmt {
        syn::Stmt::Local(local) => local,
        _ => unreachable!(),
    }
}

impl NewMethod {
    pub fn from(model: &Model, struct_def: &Struct) -> Self {
        let mut locals: Vec<_> = model
            .fields
            .iter()
            .filter_map(|field| len_init(field))
            .collect();

        let fields: Vec<_> = struct_def
            .len_vars
            .iter()
            .map(|def| def.ident.clone())
            .map(Option::unwrap)
            .map(as_len_value)
            .collect();
        let arg = match model.backend {
            Backend::Sled => {
                locals.push(sled_from_path());
                parse_quote!(path: &std::path::Path)
            }
            Backend::Trait { .. } => parse_quote!(ds: DS),
            #[cfg(test)]
            Backend::Test => unreachable!("test not used in new method"),
        };
        Self {
            locals,
            fields,
            vis: model.vis.clone(),
            arg,
        }
    }
}

#[cfg(test)]
mod tests {
    use quote::ToTokens;

    use super::*;

    #[test]
    fn has_one_local() {
        let model = Model::mock_vec();
        let struct_def = Struct::from(&model);
        let new_method = NewMethod::from(&model, &struct_def);
        assert!(new_method.locals.len() == 2);
    }

    #[test]
    fn body_is_valid_rust() {
        let model = Model::mock_vec();
        let struct_def = Struct::from(&model);
        let new_method = NewMethod::from(&model, &struct_def);

        let stmts: Vec<_> = new_method
            .locals
            .into_iter()
            .map(syn::Stmt::Local)
            .collect();
        let block = syn::Block {
            brace_token: syn::token::Brace(proc_macro2::Span::call_site()),
            stmts,
        };
        let tokens = block.to_token_stream();
        println!("{tokens}");
        assert!(syn::parse2::<syn::Block>(tokens).is_ok())
    }
}
