use ayuc_ast::{Ast, Item, ItemKind, Stmt, StmtKind};
use ayuc_diagnostic::{Diagnostic, Label};
use ayuc_resolve::def::Def;

use crate::SemanticAnalyzer;

impl SemanticAnalyzer<'_> {
    pub fn mutabilitycheck(&mut self, ast: &Ast) {
        for item in &ast.items {
            self.mc_walk_item(item);
        }
    }

    fn mc_walk_item(&mut self, item: &Item) {
        if let ItemKind::Fn(fun) = &item.kind {
            for stmt in &fun.block.children {
                self.mc_walk_stmt(stmt);
            }
        }
    }

    fn mc_walk_stmt(&mut self, stmt: &Stmt) {
        if let StmtKind::Assignment(assign) = &stmt.kind {
            let local = match self.rcx.get_name_res(assign.ident.id) {
                Def::Local(local) => local,
                _ => return,
            };

            let info = self.sess.local(local);

            if !info.mutable {
                self.dcx.emit(
                    Diagnostic::error(self.file_id, stmt.span)
                        .with_message(format!(
                            "cannot assign to immutable variable `{}`",
                            info.name
                        ))
                        .with_label(Label::help(
                            info.defined_where,
                            "this variable is immutable",
                        ))
                        .with_label(Label::primary(
                            stmt.span,
                            "cannot assign to immutable variable",
                        ))
                        .with_help(format!(
                            "consider making the variable `{name}` mutable: `let mut {name} = ...`",
                            name = info.name
                        )),
                );
            }
        }
    }
}
