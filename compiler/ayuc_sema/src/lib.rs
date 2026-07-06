pub mod callcheck;

use ayuc_ast::Ast;
use ayuc_diagnostic::DiagnosticContext;
use ayuc_resolve::resolver::ResolutionContext;
use ayuc_tyctx::TyCtx;

pub struct SemanticAnalyzer<'a> {
    rcx: &'a mut ResolutionContext,
    dcx: &'a mut DiagnosticContext,
    tyctx: &'a TyCtx,
}

impl<'a> SemanticAnalyzer<'a> {
    pub fn new(
        rcx: &'a mut ResolutionContext,
        dcx: &'a mut DiagnosticContext,
        tyctx: &'a TyCtx,
    ) -> Self {
        Self { rcx, dcx, tyctx }
    }

    pub fn analyze(
        ast: &Ast,
        rcx: &'a mut ResolutionContext,
        dcx: &'a mut DiagnosticContext,
        tyctx: &'a TyCtx,
    ) {
        let this = Self::new(rcx, dcx, tyctx);

        this.callcheck(ast);
    }
}
