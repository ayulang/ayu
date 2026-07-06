use crate::{def::Def, resolver::Resolver};

use ayuc_ast as ast;
use ayuc_diagnostic::{Diagnostic, Label};
use ayuc_id::{
    ast::NodeId,
    hir::{DefId, LocalId},
};
use ayuc_span::symbol::Symbol;

// General implementations
impl Resolver<'_> {
    pub(crate) fn resolve_names(&mut self, ast: &ast::Ast) {
        self.first_pass(ast);
        self.second_pass(ast);
    }

    fn register_def(&mut self, sym: Symbol, def_id: DefId, node_id: NodeId) {
        self.stack.register_def(sym, def_id);
        self.rcx.defs_by_node.insert(node_id, def_id);
    }

    fn register_local(&mut self, sym: Symbol, local_id: LocalId, node_id: NodeId) {
        self.stack.register_local(sym, local_id);
        self.rcx.locals_by_node.insert(node_id, local_id);
    }
}

// First pass for assigning `DefId`s to Item's `NodeId`s
// n1 = name resolution 1st pass (for avoiding conflicts with the type resolver's or 2nd pass's impl)
impl Resolver<'_> {
    fn first_pass(&mut self, ast: &ast::Ast) {
        for item in &ast.items {
            self.n1_walk_item(item);
        }
    }

    fn n1_walk_item(&mut self, item: &ast::Item) {
        let ident = match &item.kind {
            ast::ItemKind::Fn(decl) => &decl.ident,
            ast::ItemKind::ExternFn(decl) => &decl.ident,
        };
        let sym = ident.sym;

        if let Some(_def) = self.stack.lookup_top(sym) {
            self.dcx.emit(
                Diagnostic::error(self.file_id, ident.span)
                    .with_message(format!("the name `{}` is defined multiple times", sym))
                    .with_label(Label::primary(ident.span, "name is already defined")),
            );

            return;
        }

        let def_id = self.rcx.def_ids.insert(item.id);

        self.register_def(sym, def_id, item.id);
    }
}

// Second pass for resolving identifiers
// n2 = name resolution 2nd pass (for avoiding conflicts with the type resolver's or 1st pass's impl)
impl Resolver<'_> {
    fn second_pass(&mut self, ast: &ast::Ast) {
        for item in &ast.items {
            self.n2_walk_item(item);
        }
    }

    fn n2_walk_item(&mut self, item: &ast::Item) {
        if let ast::ItemKind::Fn(decl) = &item.kind {
            self.stack.enter();

            for param in &decl.parameters.parameters {
                let local_id = self.rcx.locals.insert(param.id);

                self.register_local(param.ident.sym, local_id, param.id);
            }

            for stmt in &decl.block.children {
                self.n2_walk_stmt(stmt);
            }

            self.stack.leave();
        }
    }

    fn n2_walk_stmt(&mut self, stmt: &ast::Stmt) {
        match &stmt.kind {
            ast::StmtKind::Let(decl) => {
                // Walk the expression first, so stuff like `let x = x` won't reference itself.
                self.n2_walk_expr(&decl.init);

                let local_id = self.rcx.locals.insert(stmt.id);

                self.register_local(decl.ident.sym, local_id, stmt.id);
            }

            ast::StmtKind::If(cond) => {
                self.n2_walk_expr(&cond.expr);

                self.stack.enter();

                for stmt in &cond.block.children {
                    self.n2_walk_stmt(stmt);
                }

                self.stack.leave();
            }

            ast::StmtKind::Expr(expr) => self.n2_walk_expr(expr),
            ast::StmtKind::Return(ast::ReturnStmt { expr: Some(expr) }) => self.n2_walk_expr(expr),

            ast::StmtKind::Return(_) => {}
        }
    }

    fn n2_walk_expr(&mut self, expr: &ast::Expr) {
        match &expr.kind {
            ast::ExprKind::Call(call) => {
                self.n2_walk_expr(&call.callee);

                for args in &call.args {
                    self.n2_walk_expr(args);
                }
            }

            ayuc_ast::ExprKind::Binary(bin) => {
                self.n2_walk_expr(&bin.left);
                self.n2_walk_expr(&bin.right);
            }

            ast::ExprKind::Identifier(ident) => self.n2_resolve_ident(ident),

            ast::ExprKind::Lit(_) => {}
        }
    }

    fn n2_resolve_ident(&mut self, ident: &ast::Ident) {
        if let Some(def) = self.stack.lookup(ident.sym) {
            self.rcx.name_resolutions.insert(ident.id, def);
        } else {
            self.dcx.emit(
                Diagnostic::error(self.file_id, ident.span)
                    .with_message(format!(
                        "unresolved symbol in current scope: `{}`",
                        ident.sym.as_str()
                    ))
                    .with_label(Label::primary(ident.span, "not found in current scope")),
            );

            self.rcx.name_resolutions.insert(ident.id, Def::Error);
        }
    }
}
