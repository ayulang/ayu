use ayuc_ast::{AlternateBranch, Ast, IfStmt, Item, ItemKind, Stmt, StmtKind};
use ayuc_diagnostic::{Diagnostic, Label, Recovery};

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
            StmtKind::While(r#while) => {
                for stmt in &r#while.block.children {
                    self.fc_walk_stmt(stmt, true);
                }
            }
            StmtKind::If(if_stmt) => self.fc_walk_if_stmt(if_stmt, within_loop),
            StmtKind::Break => {
                if !within_loop {
                    self.dcx.emit(
                        Diagnostic::error(self.file_id, stmt.span, Recovery::Fatal)
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

    fn fc_walk_if_stmt(&mut self, if_stmt: &IfStmt, within_loop: bool) {
        for stmt in &if_stmt.block.children {
            self.fc_walk_stmt(stmt, within_loop);
        }

        match &if_stmt.alternate {
            Some(AlternateBranch::Final(block)) => {
                for stmt in &block.children {
                    self.fc_walk_stmt(stmt, within_loop);
                }
            }
            Some(AlternateBranch::Another(if_stmt)) => self.fc_walk_if_stmt(if_stmt, within_loop),
            None => {}
        }
    }
}
