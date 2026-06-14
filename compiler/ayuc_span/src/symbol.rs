use std::{
    fmt::{Debug, Display},
    sync::LazyLock,
};

use lasso::{Spur, ThreadedRodeo};

static INTERNER: LazyLock<ThreadedRodeo> = LazyLock::new(ThreadedRodeo::default);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Symbol(Spur);

impl Symbol {
    #[inline]
    pub fn intern(s: &str) -> Self {
        INTERNER.get_or_intern(s).into()
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        INTERNER.resolve(&self.0)
    }
}

impl From<Spur> for Symbol {
    fn from(value: Spur) -> Self {
        Self(value)
    }
}

impl Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Symbol(\"{}\")", self.as_str())
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
