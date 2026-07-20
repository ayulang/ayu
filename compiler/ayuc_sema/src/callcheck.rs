use ayuc_ast::{
    AlternateBranch, Ast, CallExpr, Expr, ExprKind, IfStmt, Item, ItemKind, Stmt, StmtKind,
};
use ayuc_diagnostic::{Diagnostic, Label, Recovery};
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
            ItemKind::InlineMod(decl) => {
                for item in &decl.items {
                    self.cc_walk_item(item);
                }
            }
            ItemKind::ExternMod(_) | ItemKind::ExternFn(_) => {}
        }
    }

    fn cc_walk_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::While(r#while) => {
                self.cc_walk_expr(&r#while.expr);

                for stmt in &r#while.block.children {
                    self.cc_walk_stmt(stmt);
                }
            }
            StmtKind::Expr(expr) => self.cc_walk_expr(expr),
            StmtKind::If(if_stmt) => self.cc_walk_if_stmt(if_stmt),
            StmtKind::Loop(r#loop) => {
                for stmt in &r#loop.block.children {
                    self.cc_walk_stmt(stmt);
                }
            }
            StmtKind::Let(decl) => self.cc_walk_expr(&decl.init),
            StmtKind::Return(ret) => {
                if let Some(expr) = &ret.expr {
                    self.cc_walk_expr(expr);
                }
            }
            StmtKind::Assignment(assign) => self.cc_walk_expr(&assign.value),
            StmtKind::Break => {}
        }
    }

    fn cc_walk_if_stmt(&mut self, if_stmt: &IfStmt) {
        self.cc_walk_expr(&if_stmt.expr);

        for stmt in &if_stmt.block.children {
            self.cc_walk_stmt(stmt);
        }

        match &if_stmt.alternate {
            Some(AlternateBranch::Another(if_stmt)) => self.cc_walk_if_stmt(if_stmt),
            Some(AlternateBranch::Final(block)) => {
                for stmt in &block.children {
                    self.cc_walk_stmt(stmt);
                }
            }
            None => {}
        }
    }

    fn cc_walk_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Parenthesized(expr) => self.cc_walk_expr(expr),
            ExprKind::Call(call) => self.cc_check_call_expr(expr, call),
            ExprKind::Binary(bin) => {
                self.cc_walk_expr(&bin.left);
                self.cc_walk_expr(&bin.right);
            }
            ExprKind::Path(_) | ExprKind::Lit(_) => {}
        }
    }

    fn cc_check_call_expr(&mut self, expr: &Expr, call: &CallExpr) {
        let id = match &call.callee.kind {
            ExprKind::Path(path) => path.id,
            _ => return,
        };

        if let Def::Def(d) = self.rcx.get_name_res(id) {
            let info = self.sess.item(d);

            let provided_args = call.args.len();
            let required_args = match &info.kind {
                ayuc_session::item::ItemKind::ExternFn { n_args, .. }
                | ayuc_session::item::ItemKind::Fn { n_args, .. } => *n_args,
                ayuc_session::ItemKind::InlineMod { .. }
                | ayuc_session::ItemKind::ExternMod { .. } => return, // uncallable, maybe a diagnostic?
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
                    Diagnostic::error(self.file_id, expr.span, Recovery::Fatal)
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
