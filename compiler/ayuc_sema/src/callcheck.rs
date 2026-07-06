use ayuc_ast::{Ast, CallExpr, Expr, ExprKind, Item, ItemKind, Stmt, StmtKind};

use crate::SemanticAnalyzer;

impl SemanticAnalyzer<'_> {
    /// Checks whether all function calls pass the correct amount of arguments.
    pub fn callcheck(&self, ast: &Ast) {
        for item in &ast.items {
            self.cc_walk_item(item);
        }
    }

    fn cc_walk_item(&self, item: &Item) {
        match &item.kind {
            ItemKind::Fn(decl) => {
                for stmt in &decl.block.children {
                    self.cc_walk_stmt(stmt);
                }
            }
            ItemKind::ExternFn(_) => {}
        }
    }

    fn cc_walk_stmt(&self, stmt: &Stmt) {
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
                    self.cc_walk_expr(&expr);
                }
            }
        }
    }

    fn cc_walk_expr(&self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Call(call) => self.cc_check_call_expr(expr, call),
            ExprKind::Binary(bin) => {
                self.cc_walk_expr(&bin.left);
                self.cc_walk_expr(&bin.right);
            }
            ExprKind::Identifier(_) | ExprKind::Lit(_) => {}
        }
    }

    fn cc_check_call_expr(&self, expr: &Expr, call: &CallExpr) {
        todo!()
    }
}
