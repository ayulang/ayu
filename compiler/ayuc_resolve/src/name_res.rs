use crate::{def::Def, resolver::Resolver};

use ayuc_ast as ast;
use ayuc_diagnostic::{Diagnostic, Label};
use ayuc_id::{
    ast::NodeId,
    hir::{DefId, LocalId},
};
use ayuc_session::{self as session, ItemInfo, local::LocalInfo};
use ayuc_span::{Span, symbol::Symbol};

// General implementations
impl Resolver<'_, '_> {
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
impl Resolver<'_, '_> {
    fn first_pass(&mut self, ast: &ast::Ast) {
        for item in &ast.items {
            self.n1_walk_item(item);
        }
    }

    fn n1_walk_item(&mut self, item: &ast::Item) -> Option<DefId> {
        let ident = match &item.kind {
            ast::ItemKind::InlineMod(decl) => &decl.ident,
            ast::ItemKind::Fn(decl) => &decl.ident,
            ast::ItemKind::ExternFn(decl) => &decl.name,
        };
        let sym = ident.sym;

        if let Some(def) = self.stack.lookup(sym) {
            let mut diag = Diagnostic::error(self.file_id, ident.span)
                .with_message(format!("the name `{}` is defined multiple times", sym));

            if let Def::Def(id) = def {
                let item = self.sess.item(id);

                diag = diag.with_label(Label::help(
                    match &item.kind {
                        session::ItemKind::ExternFn { signature_span, .. }
                        | session::ItemKind::Fn { signature_span, .. }
                        | session::ItemKind::InlineMod { signature_span, .. } => *signature_span,
                    },
                    "first definition here",
                ))
            }

            diag = diag.with_label(Label::primary(ident.span, "name is already defined"));

            self.dcx.emit(diag);

            return None;
        }

        let signature_span = match &item.kind {
            ast::ItemKind::InlineMod(decl) => Span::from((item.span.start, decl.ident.span.end)),
            ast::ItemKind::Fn(decl) => Span::from((item.span.start, decl.return_ty.span.end)),
            ast::ItemKind::ExternFn(decl) => Span::from((item.span.start, decl.return_ty.span.end)),
        };

        let kind = match &item.kind {
            ast::ItemKind::Fn(decl) => session::ItemKind::Fn {
                signature_span,
                n_args: decl.parameters.parameters.len(),
            },
            ast::ItemKind::ExternFn(decl) => session::ItemKind::ExternFn {
                ffi_name: decl.ffi_name.as_ref().map(|i| i.sym),
                signature_span,
                n_args: decl.parameters.parameters.len(),
            },
            ast::ItemKind::InlineMod(decl) => {
                self.stack.enter();

                let items = decl
                    .items
                    .iter()
                    .flat_map(|item| {
                        let sym = match &item.kind {
                            ast::ItemKind::InlineMod(decl) => &decl.ident,
                            ast::ItemKind::Fn(decl) => &decl.ident,
                            ast::ItemKind::ExternFn(decl) => &decl.name,
                        }
                        .sym;

                        self.n1_walk_item(item).map(|id| (sym, id))
                    })
                    .collect();

                self.stack.leave();

                session::ItemKind::InlineMod {
                    items,
                    signature_span,
                }
            }
        };

        let def_id = self
            .sess
            .register_item(session::ItemInfo { name: sym, kind });

        self.register_def(sym, def_id, item.id);

        Some(def_id)
    }
}

// Second pass for resolving identifiers
// n2 = name resolution 2nd pass (for avoiding conflicts with the type resolver's or 1st pass's impl)
impl Resolver<'_, '_> {
    fn second_pass(&mut self, ast: &ast::Ast) {
        for item in &ast.items {
            self.n2_walk_item(item);
        }
    }

    fn n2_walk_item(&mut self, item: &ast::Item) {
        match &item.kind {
            ast::ItemKind::Fn(decl) => {
                self.stack.enter();

                for param in &decl.parameters.parameters {
                    let local_id = self.sess.register_local(LocalInfo {
                        name: param.ident.sym,
                        defined_where: param.span,
                        ty_id: param.ty.id,
                        mutable: false, // for now
                    });

                    self.register_local(param.ident.sym, local_id, param.id);
                }

                for stmt in &decl.block.children {
                    self.n2_walk_stmt(stmt);
                }

                self.stack.leave();
            }
            ast::ItemKind::InlineMod(decl) => {
                for item in &decl.items {
                    self.n2_walk_item(item);
                }
            }
            ast::ItemKind::ExternFn(_) => {}
        }
    }

    fn n2_walk_stmt(&mut self, stmt: &ast::Stmt) {
        match &stmt.kind {
            ast::StmtKind::While(r#while) => {
                self.n2_walk_expr(&r#while.expr);

                self.stack.enter();

                for stmt in &r#while.block.children {
                    self.n2_walk_stmt(stmt);
                }

                self.stack.leave();
            }
            ast::StmtKind::Loop(r#loop) => {
                self.stack.enter();

                for stmt in &r#loop.block.children {
                    self.n2_walk_stmt(stmt);
                }

                self.stack.leave();
            }
            ast::StmtKind::Assignment(assign) => {
                self.n2_resolve_ident(&assign.ident);
                self.n2_walk_expr(&assign.value);
            }
            ast::StmtKind::Let(decl) => {
                // Walk the expression first, so stuff like `let x = x` won't reference itself.
                self.n2_walk_expr(&decl.init);

                let local_id = self.sess.register_local(LocalInfo {
                    name: decl.ident.sym,
                    defined_where: stmt.span,
                    ty_id: decl.ty.id,
                    mutable: decl.mutable,
                });

                self.register_local(decl.ident.sym, local_id, stmt.id);
            }

            ast::StmtKind::If(if_stmt) => self.n2_walk_if_stmt(if_stmt),
            ast::StmtKind::Expr(expr) => self.n2_walk_expr(expr),
            ast::StmtKind::Return(ast::ReturnStmt { expr: Some(expr) }) => self.n2_walk_expr(expr),

            ast::StmtKind::Return(_) | ast::StmtKind::Break => {}
        }
    }

    fn n2_walk_if_stmt(&mut self, if_stmt: &ast::IfStmt) {
        self.n2_walk_expr(&if_stmt.expr);

        self.stack.enter();

        for stmt in &if_stmt.block.children {
            self.n2_walk_stmt(stmt);
        }

        self.stack.leave();

        match &if_stmt.alternate {
            Some(ast::AlternateBranch::Another(if_stmt)) => self.n2_walk_if_stmt(if_stmt),
            Some(ast::AlternateBranch::Final(block)) => {
                for stmt in &block.children {
                    self.n2_walk_stmt(stmt);
                }
            }
            None => {}
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

            ast::ExprKind::Binary(bin) => {
                self.n2_walk_expr(&bin.left);
                self.n2_walk_expr(&bin.right);
            }

            ast::ExprKind::Path(path) => self.n2_resolve_path(path),
            ast::ExprKind::Lit(lit) => self.n2_walk_lit(lit),
        }
    }

    fn n2_walk_lit(&mut self, lit: &ast::Literal) {
        if let ast::Literal::InterpolatedStr { segments, .. } = lit {
            for segment in segments {
                if let ast::IntlSegment::Var(ident) = segment {
                    self.n2_resolve_ident(ident);
                }
            }
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

    /// Resolves all segments of a path + the target of the whole path.
    fn n2_resolve_path(&mut self, path: &ast::Path) {
        let first = match path.segments.first() {
            Some(seg) => seg,
            _ => unreachable!(),
        };

        let ident = &first.ident;
        let def = match self.stack.lookup(ident.sym) {
            Some(mut def @ Def::Def(_)) => {
                let mut remaining = &path.segments[1..];

                self.rcx.name_resolutions.insert(first.id, def);

                while let Some(current) = remaining.first() {
                    def = match def {
                        d @ Def::Local(_) => d,
                        Def::Def(id) => self.n2_resolve_segment_in_def(current, id),
                        Def::Error => break,
                    };

                    remaining = &remaining[1..];

                    self.rcx.name_resolutions.insert(current.id, def);
                }

                def
            }
            Some(def @ Def::Local(_)) => {
                self.rcx.name_resolutions.insert(first.id, def);

                def
            }
            _ => return,
        };

        self.rcx.name_resolutions.insert(path.id, def);
    }

    fn n2_resolve_segment_in_def(&mut self, seg: &ast::PathSegment, def_id: DefId) -> Def {
        let ItemInfo {
            kind: session::ItemKind::InlineMod { items, .. },
            ..
        } = self.sess.item(def_id)
        else {
            return Def::Error;
        };

        items
            .get(&seg.ident.sym)
            .map(|id| Def::Def(*id))
            .unwrap_or(Def::Error)
    }
}
