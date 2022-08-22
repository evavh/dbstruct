use quote::format_ident;
use syn::spanned::Spanned;
use syn::{parse_quote, parse_quote_spanned};

use crate::model::{Field, Wrapper};

pub struct Accessor {
    pub vis: syn::Visibility,
    pub ident: syn::Ident,
    pub returns: syn::Type,
    pub body: syn::Block,
}

impl Accessor {
    pub fn from(field: Field) -> Self {
        let key = field.key;
        let (body, returns) = match field.wrapper {
            #[allow(unused_variables)]
            Wrapper::Vec { ty } => {
                let len_ident = format_ident!("{}_len", field.ident);
                let body = parse_quote!({
                    dbstruct::wrappers::Vec::new(self.ds.clone(), #key, self.#len_ident.clone())
                });
                let returns = parse_quote_spanned!(ty.span()=> dbstruct::wrappers::Vec<#ty, DS>);
                (body, returns)
            }
            #[allow(unused_variables)]
            Wrapper::Map { key_ty, val_ty } => {
                let body = parse_quote!({
                    dbstruct::wrappers::Map::new(self.ds.clone(), #key)
                });
                let span = key_ty.span().join(val_ty.span()).unwrap();
                let returns = parse_quote_spanned!(span=> dbstruct::wrappers::Map<#key_ty, #val_ty, DS>);
                (body, returns)
            }
            #[allow(unused_variables)]
            Wrapper::DefaultTrait { ty } => {
                let body = parse_quote!({
                    dbstruct::wrappers::DefaultTrait::new(self.ds.clone(), #key)
                });
                let returns = parse_quote_spanned!(ty.span()=> dbstruct::wrappers::DefaultTrait<#ty, DS>);
                (body, returns)
            }
            #[allow(unused_variables)]
            Wrapper::DefaultValue { ty, value } => {
                let body = parse_quote_spanned!(ty.span()=> {
                    let default_value = #value;
                    dbstruct::wrappers::DefaultValue::new(self.ds.clone(), #key, default_value)
                });
                let returns = parse_quote_spanned!(ty.span()=> dbstruct::wrappers::DefaultValue<#ty, DS>);
                (body, returns)
            }
            #[allow(unused_variables)]
            Wrapper::Option { ty } => {
                let body = parse_quote!({
                    dbstruct::wrappers::OptionValue::new(self.ds.clone(), #key)
                });
                let returns = parse_quote_spanned!(ty.span()=> dbstruct::wrappers::OptionValue<#ty, DS>);
                (body, returns)
            }
        };

        Self {
            vis: field.vis,
            ident: field.ident,
            returns,
            body,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        let field = Field {
            ident: parse_quote!(test_a),
            vis: parse_quote!(pub),
            wrapper: Wrapper::DefaultTrait {
                ty: parse_quote!(u8),
            },
            key: 1,
        };
        let _a = Accessor::from(field);
    }

    #[test]
    fn defaultval() {
        let field = Field {
            ident: parse_quote!(test_a),
            vis: parse_quote!(pub),
            wrapper: Wrapper::DefaultValue {
                ty: parse_quote!(u8),
                value: parse_quote!(5 + 12),
            },
            key: 1,
        };
        let _a = Accessor::from(field);
    }

    #[test]
    fn option() {
        let field = Field {
            ident: parse_quote!(test_a),
            vis: parse_quote!(pub),
            wrapper: Wrapper::Option {
                ty: parse_quote!(u8),
            },
            key: 1,
        };
        let _a = Accessor::from(field);
    }

    #[test]
    fn vec() {
        let field = Field {
            ident: parse_quote!(test_a),
            vis: parse_quote!(pub),
            wrapper: Wrapper::Vec {
                ty: parse_quote!(u8),
            },
            key: 1,
        };
        let _a = Accessor::from(field);
    }

    #[test]
    fn map() {
        let field = Field {
            ident: parse_quote!(test_a),
            vis: parse_quote!(pub),
            wrapper: Wrapper::Map {
                key_ty: parse_quote!(u8),
                val_ty: parse_quote!(u16),
            },
            key: 1,
        };
        let _a = Accessor::from(field);
    }
}
