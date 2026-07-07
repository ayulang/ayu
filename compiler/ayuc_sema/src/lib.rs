pub mod callcheck;

use ayuc_ast::Ast;
use ayuc_diagnostic::DiagnosticContext;
use ayuc_resolve::resolver::ResolutionContext;
use ayuc_session::Session;

pub struct SemanticAnalyzer<'a> {
    file_id: usize,
    rcx: &'a ResolutionContext,
    dcx: &'a mut DiagnosticContext,
    sess: &'a Session,
}

impl<'a> SemanticAnalyzer<'a> {
    pub fn new(
        file_id: usize,
        rcx: &'a ResolutionContext,
        dcx: &'a mut DiagnosticContext,
        sess: &'a Session,
    ) -> Self {
        Self {
            file_id,
            rcx,
            dcx,
            sess,
        }
    }

    pub fn analyze(
        ast: &Ast,
        file_id: usize,
        rcx: &'a ResolutionContext,
        dcx: &'a mut DiagnosticContext,
        sess: &'a Session,
    ) {
        let mut this = Self::new(file_id, rcx, dcx, sess);

        this.callcheck(ast);
    }
}
