use ayuc_span::{Span, symbol::Symbol};

pub struct ItemInfo {
    pub name: Symbol,
    pub kind: ItemKind,
}

pub enum ItemKind {
    Fn { signature_span: Span, n_args: usize },
    ExternFn { signature_span: Span, n_args: usize },
}
