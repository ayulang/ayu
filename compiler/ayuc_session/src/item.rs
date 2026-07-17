use std::collections::HashMap;

use ayuc_id::{ast::NodeId, hir::DefId};
use ayuc_span::{Span, symbol::Symbol};

pub struct ItemInfo {
    pub name: Symbol,
    pub kind: ItemKind,
    pub id: NodeId,
}

pub enum ItemKind {
    Fn {
        signature_span: Span,
        n_args: usize,
    },
    ExternFn {
        ffi_name: Option<Symbol>,
        signature_span: Span,
        n_args: usize,
    },
    InlineMod {
        signature_span: Span,
        items: HashMap<Symbol, DefId>,
    },
    ExternMod {
        ffi_name: Option<Symbol>,
        signature_span: Span,
        items: HashMap<Symbol, DefId>,
    },
}
