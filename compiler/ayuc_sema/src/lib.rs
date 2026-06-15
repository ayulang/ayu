use ayuc_hir::PackageId;
use ayuc_tyctx::TyCtx;

use crate::{
    pass::{build_scope::BuildScopePass, resolve_names::ResolveNamesPass},
    scope::ScopeCtx,
};

pub mod pass;
pub mod scope;

pub struct SemanticAnalyzer<'a> {
    pub ty_ctx: &'a mut TyCtx,
    pub scope_ctx: &'a mut ScopeCtx,
}

impl<'a> SemanticAnalyzer<'a> {
    pub fn new(ty_ctx: &'a mut TyCtx, scope_ctx: &'a mut ScopeCtx) -> Self {
        Self { ty_ctx, scope_ctx }
    }

    pub fn run_on(&'a mut self, package_id: PackageId) {
        let package = self.ty_ctx.package(package_id);

        BuildScopePass::new(self.scope_ctx).run(package);
        ResolveNamesPass::new(self.scope_ctx).run(package);
    }
}
