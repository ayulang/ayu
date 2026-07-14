use ayuc_ast::{Ast, Item, ItemKind, Stmt, StmtKind};
use ayuc_diagnostic::{Diagnostic, Label};

use crate::SemanticAnalyzer;

impl SemanticAnalyzer<'_> {
    pub fn flowcheck(&mut self, ast: &Ast) {
        for item in &ast.items {
            self.fc_walk_item(item);
        }
    }

    fn fc_walk_item(&mut self, item: &Item) {
        if let ItemKind::Fn(fun) = &item.kind {
            for stmt in &fun.block.children {
                self.fc_walk_stmt(stmt, false);
            }
        }
    }

    fn fc_walk_stmt(&mut self, stmt: &Stmt, within_loop: bool) {
        match &stmt.kind {
            StmtKind::Loop(r#loop) => {
                for stmt in &r#loop.block.children {
                    self.fc_walk_stmt(stmt, true);
                }
            }
            StmtKind::If(cond) => {
                for stmt in &cond.block.children {
                    self.fc_walk_stmt(stmt, within_loop);
                }
            }
            StmtKind::Break => {
                if !within_loop {
                    self.dcx.emit(
                        Diagnostic::error(self.file_id, stmt.span)
                            .with_message("`break` outside of a loop")
                            .with_label(Label::primary(stmt.span, "outside of a loop"))
                            .with_help("maybe you confused `break` with `return`"),
                    );
                }
            }
            StmtKind::Expr(_)
            | StmtKind::Let(_)
            | StmtKind::Return(_)
            | StmtKind::Assignment(_) => {}
        }
    }
}
