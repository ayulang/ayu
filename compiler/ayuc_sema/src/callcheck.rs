use ayuc_ast::{Ast, CallExpr, Expr, ExprKind, Item, ItemKind, Stmt, StmtKind};
use ayuc_diagnostic::{Diagnostic, Label};
use ayuc_resolve::def::Def;

use crate::SemanticAnalyzer;

impl SemanticAnalyzer<'_> {
    /// Checks whether all function calls pass the correct amount of arguments.
    pub fn callcheck(&mut self, ast: &Ast) {
        for item in &ast.items {
            self.cc_walk_item(item);
        }
    }

    fn cc_walk_item(&mut self, item: &Item) {
        match &item.kind {
            ItemKind::Fn(decl) => {
                for stmt in &decl.block.children {
                    self.cc_walk_stmt(stmt);
                }
            }
            ItemKind::ExternFn(_) => {}
        }
    }

    fn cc_walk_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Expr(expr) => self.cc_walk_expr(expr),
            StmtKind::If(cond) => {
                self.cc_walk_expr(&cond.expr);

                for stmt in &cond.block.children {
                    self.cc_walk_stmt(stmt);
                }
            }
            StmtKind::Let(decl) => self.cc_walk_expr(&decl.init),
            StmtKind::Return(ret) => {
                if let Some(expr) = &ret.expr {
                    self.cc_walk_expr(expr);
                }
            }
        }
    }

    fn cc_walk_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Call(call) => self.cc_check_call_expr(expr, call),
            ExprKind::Binary(bin) => {
                self.cc_walk_expr(&bin.left);
                self.cc_walk_expr(&bin.right);
            }
            ExprKind::Identifier(_) | ExprKind::Lit(_) => {}
        }
    }

    fn cc_check_call_expr(&mut self, expr: &Expr, call: &CallExpr) {
        let id = match &call.callee.kind {
            ExprKind::Identifier(ident) => ident.id,
            _ => return,
        };

        if let Def::Def(d) = self.rcx.get_name_res(id) {
            let info = self.sess.item(d);

            let provided_args = call.args.len();
            let required_args = match &info.kind {
                ayuc_session::item::ItemKind::ExternFn { n_args }
                | ayuc_session::item::ItemKind::Fn { n_args } => *n_args,
            };

            if provided_args != required_args {
                let takes = format!(
                    "{} parameter{}",
                    required_args,
                    if required_args == 1 { "" } else { "s" }
                );

                let provided = format!(
                    "{} parameter{}",
                    provided_args,
                    if provided_args == 1 { "" } else { "s" }
                );

                self.dcx.emit(
                    Diagnostic::error(self.file_id, expr.span)
                        .with_message(format!(
                            "function `{}` takes {takes}, but {provided} were provided",
                            info.name
                        ))
                        .with_label(Label::primary(
                            expr.span,
                            format!(
                                "{} parameter{} were provided",
                                call.args.len(),
                                if call.args.len() != 1 { "s" } else { "" }
                            ),
                        )),
                );
            }
        }
    }
}
