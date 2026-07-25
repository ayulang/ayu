use std::fmt::Display;

use ayuc_id::TyId;

use crate::{interner::TypeInterner, ty::TyKind};

/// A wrapper around a [TyKind] that provides an implementation of the [Display] trait.
/// This is necessary because a [TyKind] may hold other [TyId]s, which need a [TypeInterner] to be resolved.
pub struct FormatTy<'a> {
    ty: &'a TyKind,
    interner: &'a TypeInterner,
}

impl<'a> FormatTy<'a> {
    pub fn new(ty: &'a TyKind, interner: &'a TypeInterner) -> Self {
        Self { ty, interner }
    }
}

impl Display for FormatTy<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.ty {
            TyKind::Prim(prim) => write!(f, "{prim}")?,
            TyKind::Fn(params, ret) => {
                write!(f, "(")?;

                for (i, param) in params.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }

                    let param = self.interner.get(*param);

                    write!(f, "{}", param.format(self.interner))?;
                }

                write!(f, ") -> ")?;

                let ret = self.interner.get(*ret);

                write!(f, "{}", ret.format(self.interner))?;
            }
            TyKind::Tuple(elements) => {
                write!(f, "(")?;

                for (i, element) in elements.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }

                    let param = self.interner.get(*element);

                    write!(f, "{}", param.format(self.interner))?;
                }

                write!(f, ")")?;
            }
        }

        Ok(())
    }
}

pub trait FormatExt {
    fn format<'a>(&'a self, interner: &'a TypeInterner) -> FormatTy<'a>;
}

impl FormatExt for TyId {
    fn format<'a>(&'a self, interner: &'a TypeInterner) -> FormatTy<'a> {
        FormatTy::new(interner.get(*self), interner)
    }
}
