use ayuc_span::symbol::Symbol;

pub struct ItemInfo {
    pub name: Symbol,
    pub kind: ItemKind,
}

pub enum ItemKind {
    Fn { n_args: usize },
    ExternFn { n_args: usize },
}
