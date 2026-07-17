use ayuc_ast as ast;
use ayuc_diagnostic::{Diagnostic, Label};
use ayuc_resolve::def::Def;
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
            ast::StmtKind::Loop(r#loop) => {
                for stmt in &r#loop.block.children {
                    self.tc_walk_stmt(stmt);
                }
            }
            ast::StmtKind::While(r#while) => {
                for stmt in &r#while.block.children {
                    self.tc_walk_stmt(stmt);
                }
            }
            ast::StmtKind::Assignment(assign) => self.tc_check_assign_stmt(stmt, assign),
            ast::StmtKind::Let(decl) => self.tc_check_let_stmt(stmt, decl),
            ast::StmtKind::Expr(_)
            | ast::StmtKind::If(_)
            | ast::StmtKind::Return(_)
            | ast::StmtKind::Break => {}
        }
    }

    fn tc_check_assign_stmt(&mut self, stmt: &ast::Stmt, assign: &ast::AssignStmt) {
        let local = match self.rcx.get_name_res(assign.ident.id) {
            Def::Local(local) => local,
            _ => return,
        };

        let info = self.sess.local(local);

        let ty = self.rcx.ty_res(info.id);
        let expr_ty = self.rcx.ty_res(assign.value.id);

        if ty.is_error() || expr_ty.is_error() {
            return;
        }

        if ty != expr_ty {
            self.dcx.emit(
                Diagnostic::error(self.file_id, stmt.span)
                    .with_message(format!("expected type {}, got type {}", ty, expr_ty))
                    .with_label(Label::help(
                        info.defined_where,
                        format!("this is of type {}", ty),
                    ))
                    .with_label(Label::primary(
                        assign.value.span,
                        format!("this is of type {}", expr_ty),
                    )),
            );
        }
    }

    fn tc_check_let_stmt(&mut self, stmt: &ast::Stmt, decl: &ast::LetStmt) {
        let decl_ty = self.rcx.ty_res(stmt.id);
        let expr_ty = self.rcx.ty_res(decl.init.id);

        if decl_ty.is_error() || expr_ty.is_error() {
            return;
        }

        if expr_ty != decl_ty {
            self.dcx.emit(
                Diagnostic::error(self.file_id, stmt.span)
                    .with_message(format!("expected type {}, got type {}", decl_ty, expr_ty))
                    .with_label(Label::help(
                        Span::from((
                            stmt.span.start,
                            match &decl.ty {
                                Some(ty) => ty.span.end,
                                None => decl.ident.span.end,
                            },
                        )),
                        format!("this is of type {}", decl_ty),
                    ))
                    .with_label(Label::primary(
                        decl.init.span,
                        format!("this is of type {}", expr_ty),
                    )),
            );
        }
    }
}
