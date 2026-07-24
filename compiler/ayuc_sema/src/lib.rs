pub(crate) mod flow;
pub(crate) mod general;
pub(crate) mod mutability;
pub(crate) mod typecheck;

use ayuc_ast::Ast;
use ayuc_ast_visit::visitor::Visitor;
use ayuc_diagnostic::DiagnosticContext;
use ayuc_resolve::resolver::ResolutionContext;
use ayuc_session::Session;

use crate::{
    flow::FlowAnalysisPhase, general::GeneralPhase, mutability::MutabilityAnalysisPhase,
    typecheck::TypeCheckingPhase,
};

macro_rules! run_phase {
    ($ast:ident, $phase:expr) => {{
        let mut phase = $phase;

        phase.visit_ast($ast);
    }};
}

pub struct SemanticAnalyzer;

impl SemanticAnalyzer {
    pub fn analyze(
        ast: &Ast,
        file_id: usize,
        rcx: &ResolutionContext,
        dcx: &mut DiagnosticContext,
        sess: &Session,
    ) {
        run_phase!(ast, GeneralPhase::new(dcx, file_id));
        run_phase!(ast, TypeCheckingPhase::new(dcx, rcx, sess, file_id));
        run_phase!(ast, MutabilityAnalysisPhase::new(dcx, rcx, sess, file_id));
        run_phase!(ast, FlowAnalysisPhase::new(dcx, file_id))
    }
}
