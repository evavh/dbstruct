use std::collections::HashSet;

use super::attribute::{BackendOption, Options};
use super::{attribute, Field};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Multiple backends specified")]
    MultipleBackends,
    #[error("No database backend specified for the struct")]
    NoBackendSpecified,
    #[error("No database backend specified for the struct")]
    MissesTraits {
        backend: Backend,
        missing: HashSet<ExtraBound>,
    },
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum ExtraBound {
    Atomic,
    Orderd,
}

#[derive(Debug, Clone)]
pub enum Backend {
    Sled,
    Trait {
        bounds: Vec<ExtraBound>,
    },
    #[cfg(test)]
    Test,
}

impl Backend {
    pub fn try_from(options: &[attribute::Options], fields: &[Field]) -> Result<Backend, Error> {
        let mut backends = options.iter().filter_map(|opt| match opt {
            Options::Backend(b) => Some(b),
            _ => None,
        });

        let backend = match (backends.next(), backends.next()) {
            (None, _) => return Err(Error::NoBackendSpecified),
            (Some(b), None) => *b,
            (Some(_), Some(_)) => return Err(Error::MultipleBackends),
        };

        let backend = match backend {
            BackendOption::Trait => {
                return Ok(Backend::Trait {
                    bounds: fields
                        .iter()
                        .map(|f| f.wrapper.needed_traits().into_iter())
                        .flatten()
                        .collect(),
                })
            }
            BackendOption::Sled => Backend::Sled,
            #[cfg(test)]
            BackendOption::Test => Backend::Test,
        };

        for field in fields {
            let needed = field.wrapper.needed_traits();
            let missing: HashSet<_> = needed.difference(&backend.traits()).copied().collect();
            if !missing.is_empty() {
                return Err(Error::MissesTraits { backend, missing });
            }
        }

        Ok(backend)
    }

    fn traits(&self) -> HashSet<ExtraBound> {
        use ExtraBound::*;
        match self {
            Backend::Sled => vec![Atomic, Orderd].into_iter(),
            Backend::Trait { .. } => unreachable!("should never be called when backend is Trait"),
            #[cfg(test)]
            Backend::Test => vec![].into_iter(),
        }
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use attribute::Wrapper;
    use syn::parse_quote;
    use ExtraBound::*;

    use super::*;

    #[test]
    fn error_on_unsupported_backend() {
        let options = [Options::Backend(BackendOption::Test)];
        let fields = [Field {
            ident: parse_quote!(test_a),
            vis: parse_quote!(pub),
            wrapper: Wrapper::Vec {
                ty: parse_quote!(u8),
            },
            key: 1,
        }];
        let err = Backend::try_from(&options, &fields).unwrap_err();
        match err {
            Error::MissesTraits { missing, .. } => {
                let correct = [Atomic, Orderd].into_iter().collect();
                assert_eq!(missing, correct);
            }
            _ => unreachable!("expected error missingtraits got: {err:?}"),
        }
    }

    #[test]
    fn supported_backend() {
        let options = [Options::Backend(BackendOption::Sled)];
        let fields = [Field {
            ident: parse_quote!(test_a),
            vis: parse_quote!(pub),
            wrapper: Wrapper::Vec {
                ty: parse_quote!(u8),
            },
            key: 1,
        }];
        let backend = Backend::try_from(&options, &fields).unwrap();
        assert!(matches!(backend, Backend::Sled));
    }

    #[test]
    fn reject_double_backend() {
        let options = [
            Options::Backend(BackendOption::Sled),
            Options::Backend(BackendOption::Sled),
        ];
        let fields = [Field {
            ident: parse_quote!(test_a),
            vis: parse_quote!(pub),
            wrapper: Wrapper::Vec {
                ty: parse_quote!(u8),
            },
            key: 1,
        }];
        let err = Backend::try_from(&options, &fields).unwrap_err();
        assert!(matches!(err, Error::MultipleBackends));
    }
}
