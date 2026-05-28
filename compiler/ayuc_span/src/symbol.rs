use std::sync::LazyLock;

use lasso::{Spur, ThreadedRodeo};

static INTERNER: LazyLock<ThreadedRodeo> = LazyLock::new(ThreadedRodeo::default);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
