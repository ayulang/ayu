pub(crate) mod callcheck;
pub(crate) mod flow;
pub(crate) mod general;
pub(crate) mod mutability;
pub(crate) mod typecheck;

use ayuc_ast::Ast;
use ayuc_ast_visit::visitor::Visitor;
use ayuc_diagnostic::DiagnosticContext;
use ayuc_resolve::resolver::ResolutionContext;
use ayuc_session::Session;

use crate::general::GeneralPhase;

macro_rules! run_phase {
    ($ast:ident, $phase:expr) => {{
        let mut phase = $phase;

        phase.visit_ast($ast);
    }};
}

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

        run_phase!(ast, GeneralPhase::new(&mut this.dcx, this.file_id));

        this.callcheck(ast);
        this.typecheck(ast);
        this.mutabilitycheck(ast);
        this.flowcheck(ast);
    }
}
