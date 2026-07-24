use ayuc_ast::AssignStmt;
use ayuc_ast_visit::{visitor::Visitor, walkable::Walkable};
use ayuc_diagnostic::{Diagnostic, DiagnosticContext, Label, Recovery};
use ayuc_resolve::{def::Def, resolver::ResolutionContext};
use ayuc_session::Session;
use ayuc_span::Span;

pub struct MutabilityAnalysisPhase<'a> {
    dcx: &'a mut DiagnosticContext,
    rcx: &'a ResolutionContext,
    sess: &'a Session,
    file_id: usize,

    stmt_span: Option<Span>,
}

impl<'a> MutabilityAnalysisPhase<'a> {
    pub fn new(
        dcx: &'a mut DiagnosticContext,
        rcx: &'a ResolutionContext,
        sess: &'a Session,
        file_id: usize,
    ) -> Self {
        Self {
            dcx,
            rcx,
            sess,
            file_id,
            stmt_span: None,
        }
    }
}

impl Visitor<'_> for MutabilityAnalysisPhase<'_> {
    fn visit_stmt(&mut self, stmt: &'_ ayuc_ast::Stmt) {
        let old = self.stmt_span.replace(stmt.span);

        stmt.walk(self);

        self.stmt_span = old;
    }

    fn visit_assign_stmt(&mut self, assign_stmt: &'_ AssignStmt) {
        let stmt_span = self.stmt_span.unwrap();

        let local = match self.rcx.get_name_res(assign_stmt.ident.id) {
            Def::Local(local) => local,
            _ => return assign_stmt.walk(self),
        };

        let info = self.sess.local(local);

        if !info.mutable {
            self.dcx.emit(
                Diagnostic::error(self.file_id, stmt_span, Recovery::Fatal)
                    .with_message(format!(
                        "cannot assign to immutable variable `{}`",
                        info.name
                    ))
                    .with_label(Label::help(
                        info.defined_where,
                        "this variable is immutable",
                    ))
                    .with_label(Label::primary(
                        stmt_span,
                        "cannot assign to immutable variable",
                    ))
                    .with_help(format!(
                        "consider making the variable `{name}` mutable: `let mut {name} = ...`",
                        name = info.name
                    )),
            );
        }

        assign_stmt.walk(self)
    }
}
