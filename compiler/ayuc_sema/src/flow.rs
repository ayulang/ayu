use ayuc_ast::{LoopStmt, Stmt, WhileStmt};
use ayuc_ast_visit::{visitor::Visitor, walkable::Walkable};
use ayuc_diagnostic::{Diagnostic, DiagnosticContext, Label, Recovery};
use ayuc_span::Span;

pub struct FlowAnalysisPhase<'a> {
    dcx: &'a mut DiagnosticContext,
    file_id: usize,

    flow_depth: usize,
    current_stmt_span: Option<Span>,
}

impl<'a> FlowAnalysisPhase<'a> {
    pub fn new(dcx: &'a mut DiagnosticContext, file_id: usize) -> Self {
        Self {
            dcx,
            file_id,
            flow_depth: 0,
            current_stmt_span: None,
        }
    }
}

impl Visitor<'_> for FlowAnalysisPhase<'_> {
    fn visit_stmt(&mut self, stmt: &'_ Stmt) {
        let old_span = self.current_stmt_span.replace(stmt.span);

        stmt.walk(self);

        self.current_stmt_span = old_span;
    }

    fn visit_loop_stmt(&mut self, loop_stmt: &'_ LoopStmt) {
        self.flow_depth += 1;

        loop_stmt.walk(self);

        self.flow_depth -= 1;
    }

    fn visit_while_stmt(&mut self, while_stmt: &'_ WhileStmt) {
        self.flow_depth += 1;

        while_stmt.walk(self);

        self.flow_depth -= 1;
    }

    fn visit_break_stmt(&mut self) {
        let Some(span) = self.current_stmt_span else {
            return;
        };

        if self.flow_depth == 0 {
            self.dcx.emit(
                Diagnostic::error(self.file_id, span, Recovery::Fatal)
                    .with_message("`break` statement outside of loop")
                    .with_label(Label::primary(span, "cannot `break` outside of a loop"))
                    .with_help("you might have confused `break` with `return`"),
            );
        }
    }
}
