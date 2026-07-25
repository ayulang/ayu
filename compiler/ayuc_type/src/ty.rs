use std::fmt::Display;

use ayuc_id::TyId;

use crate::{format::FormatTy, interner::TypeInterner};

/// A kind of type. This can be a single primitive, a tuple, or a function.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TyKind {
    Prim(PrimTy),
    Tuple(Vec<TyId>),
    Fn(Vec<TyId>, TyId),
}

impl TyKind {
    #[inline]
    pub fn is_unit(&self) -> bool {
        matches!(self, TyKind::Tuple(vec) if vec.is_empty())
    }

    #[inline]
    pub fn format<'a>(&'a self, interner: &'a TypeInterner) -> FormatTy<'a> {
        FormatTy::new(self, interner)
    }
}

/// A primitive or built-in type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrimTy {
    Bool,
    Int,
    Str,
}

impl Display for PrimTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool => write!(f, "bool"),
            Self::Int => write!(f, "int"),
            Self::Str => write!(f, "str"),
        }
    }
}

pub trait IsUnitExt {
    fn is_unit(&self, interner: &TypeInterner) -> bool;
}

impl IsUnitExt for TyId {
    #[inline]
    fn is_unit(&self, interner: &TypeInterner) -> bool {
        interner.get(*self).is_unit()
    }
}
