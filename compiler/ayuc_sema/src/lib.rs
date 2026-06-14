use ayuc_tyctx::TyCtx;

use crate::scope::ScopeCtx;

pub mod scope;

pub struct SemanticAnalyzer<'a> {
    pub ty_ctx: &'a mut TyCtx,
    pub scope_ctx: ScopeCtx,
}

impl<'a> SemanticAnalyzer<'a> {
    pub fn new(ty_ctx: &'a mut TyCtx) -> Self {
        Self {
            ty_ctx,
            scope_ctx: ScopeCtx::default(),
        }
    }
}
