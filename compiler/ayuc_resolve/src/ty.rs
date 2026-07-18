use std::fmt::Display;

use ayuc_span::symbol::Symbol;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    Unit,
    Prim(PrimTy),
    Fn(Vec<Ty>, Box<Ty>),
    Error,
}

impl Ty {
    pub fn is_error(&self) -> bool {
        match self {
            Self::Error => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimTy {
    Boolean,
    Integer,
    Str,
}

impl Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Prim(ty) => match ty {
                PrimTy::Boolean => write!(f, "bool")?,
                PrimTy::Integer => write!(f, "int")?,
                PrimTy::Str => write!(f, "str")?,
            },
            Self::Unit => write!(f, "()")?,
            Self::Fn(params, ret) => {
                write!(f, "(")?;

                for (i, params) in params.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{}", params)?;
                }

                write!(f, ") -> {ret}")?;
            }
            Self::Error => unreachable!(),
        };

        Ok(())
    }
}

impl PrimTy {
    pub fn from_name(sym: Symbol) -> Option<Self> {
        let prim = match sym.as_str() {
            "bool" => Self::Boolean,
            "int" => Self::Integer,
            "str" => Self::Str,
            _ => return None,
        };

        Some(prim)
    }
}
