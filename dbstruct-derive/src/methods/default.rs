use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Ident, Type};

pub fn methods(
    ident: &proc_macro2::Ident,
    full_type: &Type,
    key: &str,
    default_val: &TokenStream,
) -> TokenStream {
    let setter = setter(ident, full_type, &key);
    let getter = getter(ident, full_type, &key, &default_val);
    let update = update(ident, full_type, &key, &default_val);
    let compare_and_swap = compare_and_swap(ident, full_type, &key, &default_val);

    quote!(
        #setter
        #getter
        #update
        #compare_and_swap
    )
}

fn setter(ident: &proc_macro2::Ident, full_type: &Type, key: &str) -> TokenStream {
    let setter = Ident::new(&format!("set_{}", ident), ident.span());
    let span = ident.span();

    quote_spanned! {span=>
        #[allow(dead_code)]
        pub fn #setter(&self, position: &#full_type) -> std::result::Result<(), dbstruct::Error> {
            let bytes = bincode::serialize(position)
                .map_err(dbstruct::Error::Serializing)?;
            self.tree.insert(#key, bytes)?;
            Ok(())
        }
    }
}

fn getter(
    ident: &proc_macro2::Ident,
    full_type: &Type,
    key: &str,
    default_val: &TokenStream,
) -> TokenStream {
    let getter = ident.clone();
    let span = ident.span();

    quote_spanned! {span=>
        /// getter for #ident
        /// # Errors
        /// TODO
        #[allow(dead_code)]
        pub fn #getter(&self) -> std::result::Result<#full_type, dbstruct::Error> {
            let default_val = #default_val;
            match self.tree.get(#key)? {
                Some(bytes) => Ok(bincode::deserialize(&bytes).map_err(dbstruct::Error::DeSerializing)?),
                None => Ok(default_val),
            }
        }
    }
}

fn update(
    ident: &proc_macro2::Ident,
    full_type: &Type,
    key: &str,
    default_val: &TokenStream,
) -> TokenStream {
    let update = Ident::new(&format!("update_{}", ident), ident.span());
    let span = full_type.span();

    quote_spanned! {span=>
        /// # Errors
        /// returns an error incase de or re-serializing failed, in which case the
        /// value of the member in the array will not have changed.
        #[allow(dead_code)]
        pub fn #update(&self, op: impl FnMut(#full_type) -> #full_type + Clone)
            -> std::result::Result<(), dbstruct::Error> {
            let default_val = #default_val;

            let mut res = Ok(());
            let update = |old: Option<&[u8]>| {
                match old {
                    None => {
                        let new = op.clone()(default);
                        match bincode::serialize(&new) {
                            Ok(new_bytes) => Some(new_bytes),
                            Err(e) => {
                                res = Err(dbstruct::Error::Serializing(e));
                                None
                            }
                        }
                    }
                    Some(old) => match bincode::deserialize(old) {
                        Err(e) => {
                            res = Err(dbstruct::Error::DeSerializing(e));
                            Some(old.to_vec())
                        }
                        Ok(v) => {
                            let new = op.clone()(v);
                            match bincode::serialize(&new) {
                                Ok(new_bytes) => Some(new_bytes),
                                Err(e) => {
                                    res = Err(dbstruct::Error::Serializing(e));
                                    Some(old.to_vec())
                                }
                            }
                        }
                    }
                }
            };

            self.tree.update_and_fetch(#key, update)?;
            Ok(())
        }
    }
}

fn compare_and_swap(
    ident: &proc_macro2::Ident,
    full_type: &Type,
    key: &str,
    default_val: &TokenStream,
) -> TokenStream {
    let compare_and_swap = Ident::new(&format!("compare_and_swap_{}", ident), ident.span());
    let span = full_type.span();

    quote_spanned! {span=>
        #[allow(dead_code)]
        pub fn #compare_and_swap(&self, old: #full_type, new: #full_type)
            -> std::result::Result<
                std::result::Result<(), dbstruct::CompareAndSwapError<#full_type>>,
            dbstruct::Error> {

            // The default value is encoded as no value in the db. If the user is
            // comparing agains the old vale change the call in the array
            let old = if old == #default_val {
                None,
            } else {
                let old = bincode::serialize(&old).map_err(dbstruct::Error::Serializing)?;
            }

            // I save the default as None not to save space but keep initialization
            // fast, otherwise the default value would need to be written for each
            // dbstruct member. Therefore we do not need to encode the new as None if
            // it is the default
            let new = bincode::serialize(&new).map_err(dbstruct::Error::Serializing)?;
            Ok(match self.tree.compare_and_swap(#key, Some(old), Some(new))? {
                Ok(()) => Ok(()),
                Err(e) => Err(e.try_into()?),
            })
        }
    }
}
