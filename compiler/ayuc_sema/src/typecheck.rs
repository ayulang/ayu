use ayuc_ast as ast;
use ayuc_diagnostic::{Diagnostic, Label};
use ayuc_resolve as resolve;
use ayuc_span::Span;

use crate::SemanticAnalyzer;

impl SemanticAnalyzer<'_> {
    pub fn typecheck(&mut self, ast: &ast::Ast) {
        for item in &ast.items {
            if let ast::ItemKind::Fn(decl) = &item.kind {
                for stmt in &decl.block.children {
                    self.tc_walk_stmt(stmt);
                }
            }
        }
    }

    fn tc_walk_stmt(&mut self, stmt: &ast::Stmt) {
        match &stmt.kind {
            ast::StmtKind::Let(decl) => self.tc_check_let_stmt(stmt, decl),
            ast::StmtKind::Expr(_) | ast::StmtKind::If(_) | ast::StmtKind::Return(_) => {}
        }
    }

    fn tc_check_let_stmt(&mut self, stmt: &ast::Stmt, decl: &ast::LetStmt) {
        let decl_ty = self.rcx.get_ty_res(decl.ty.id);
        let expr_ty = self.tc_type_of_expr(&decl.init);

        if decl_ty == resolve::Ty::Error || expr_ty == resolve::Ty::Error {
            return;
        }

        if expr_ty != decl_ty {
            self.dcx.emit(
                Diagnostic::error(self.file_id, stmt.span)
                    .with_message(format!(
                        "expected type {}, got type {}",
                        decl_ty.get_name(),
                        expr_ty.get_name()
                    ))
                    .with_label(Label::help(
                        Span::from((stmt.span.start, decl.ty.span.end)),
                        format!("this is of type {}", decl_ty.get_name()),
                    ))
                    .with_label(Label::primary(
                        decl.init.span,
                        format!("this is of type {}", expr_ty.get_name()),
                    )),
            );
        }
    }

    fn tc_type_of_expr(&self, expr: &ast::Expr) -> resolve::Ty {
        match &expr.kind {
            ast::ExprKind::Lit(lit) => match lit {
                ast::Literal::Integer { .. } => resolve::Ty::Prim(resolve::PrimTy::Integer),
                ast::Literal::Str { .. } => resolve::Ty::Prim(resolve::PrimTy::Str),
            },
            _ => todo!(),
        }
    }
}
