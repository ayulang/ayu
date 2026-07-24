use std::collections::HashMap;

use ayuc_id::{ast::NodeId, hir::DefId};
use ayuc_span::{Span, symbol::Symbol};

#[derive(PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
}

pub struct ItemInfo {
    pub name: Symbol,
    pub kind: ItemKind,
    pub id: NodeId,
    pub vis: Visibility,
}

impl ItemInfo {
    pub fn signature_span(&self) -> Span {
        match &self.kind {
            ItemKind::ExternFn { signature_span, .. }
            | ItemKind::Fn { signature_span, .. }
            | ItemKind::InlineMod { signature_span, .. }
            | ItemKind::ExternMod { signature_span, .. } => *signature_span,
        }
    }
}

pub enum ItemKind {
    Fn {
        signature_span: Span,
    },
    ExternFn {
        ffi_name: Option<Symbol>,
        signature_span: Span,
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
