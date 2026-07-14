use ayuc_ast as ast;
use ayuc_diagnostic::{Diagnostic, Label};
use ayuc_resolve::{self as resolve, def::Def};
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
            ast::StmtKind::Assignment(assign) => self.tc_check_assign_stmt(stmt, assign),
            ast::StmtKind::Let(decl) => self.tc_check_let_stmt(stmt, decl),
            ast::StmtKind::Expr(_) | ast::StmtKind::If(_) | ast::StmtKind::Return(_) => {}
        }
    }

    fn tc_check_assign_stmt(&mut self, stmt: &ast::Stmt, assign: &ast::AssignStmt) {
        let local = match self.rcx.get_name_res(assign.ident.id) {
            Def::Local(local) => local,
            _ => return,
        };

        let info = self.sess.local(local);

        let ty = self.rcx.get_ty_res(info.ty_id);
        let expr_ty = self.tc_type_of_expr(&assign.value);

        if ty == resolve::Ty::Error || expr_ty == resolve::Ty::Error {
            return;
        }

        if ty != expr_ty {
            self.dcx.emit(
                Diagnostic::error(self.file_id, stmt.span)
                    .with_message(format!(
                        "expected type {}, got type {}",
                        ty.get_name(),
                        expr_ty.get_name()
                    ))
                    .with_label(Label::help(
                        info.defined_where,
                        format!("this is of type {}", ty.get_name()),
                    ))
                    .with_label(Label::primary(
                        assign.value.span,
                        format!("this is of type {}", expr_ty.get_name()),
                    )),
            );
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
        use ast::{Literal as Lit, Operator as Op};
        use resolve::{PrimTy, Ty};

        match &expr.kind {
            ast::ExprKind::Lit(lit) => match lit {
                Lit::Bool { .. } => Ty::Prim(PrimTy::Boolean),
                Lit::Integer { .. } => Ty::Prim(PrimTy::Integer),
                Lit::Str { .. } | Lit::InterpolatedStr { .. } => Ty::Prim(PrimTy::Str),
            },
            ast::ExprKind::Binary(bin) => match bin.operator {
                Op::Add | Op::Minus => Ty::Prim(PrimTy::Integer), // probably... for now
                Op::EqualsEquals
                | Op::Gt
                | Op::GtOrEqual
                | Op::Lt
                | Op::LtOrEqual
                | Op::NotEquals => Ty::Prim(PrimTy::Boolean),
            },
            _ => Ty::Error,
        }
    }
}
