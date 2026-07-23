use std::collections::VecDeque;

use crate::{
    Ty,
    def::Def,
    resolver::Resolver,
    ty::{PrimTy, TyKind},
};

use ayuc_ast::{self as ast, AlternateBranch, ExprKind, IfStmt, Literal, Operator, PatKind};
use ayuc_diagnostic::{Diagnostic, Label, Recovery};
use ayuc_id::TyId;
use ayuc_span::Span;

impl Resolver<'_, '_> {
    pub(crate) fn resolve_types(&mut self, ast: &ast::Ast) {
        for item in &ast.items {
            self.tr_walk_item(item);
        }
    }

    fn tr_walk_item(&mut self, item: &ast::Item) {
        match &item.kind {
            ast::ItemKind::InlineMod(ast::ModItem { items, .. })
            | ast::ItemKind::ExternMod(ast::ExternModItem { items, .. }) => {
                for item in items {
                    self.tr_walk_item(item);
                }
            }
            ast::ItemKind::Fn(fun) => {
                let mut parameters = Vec::with_capacity(fun.parameters.parameters.len());

                for param in &fun.parameters.parameters {
                    let id = self.tr_resolve_ty(&param.ty);

                    self.rcx.tys_by_node.insert(param.id, id);

                    parameters.push(self.rcx.ty(id).clone());
                }

                let return_ty_id = self.tr_resolve_ty(&fun.return_ty);
                let kind = TyKind::Fn(parameters, Box::new(self.rcx.ty(return_ty_id).clone()));
                let id = self
                    .rcx
                    .ty_resolutions
                    .insert_with_key(|key| Ty { id: key, kind });

                self.rcx.tys_by_node.insert(item.id, id);

                for stmt in &fun.block.children {
                    self.tr_walk_stmt(stmt);
                }
            }
            ast::ItemKind::ExternFn(extern_fun) => {
                let mut parameters = Vec::with_capacity(extern_fun.parameters.parameters.len());

                for param in &extern_fun.parameters.parameters {
                    let id = self.tr_resolve_ty(&param.ty);

                    self.rcx.tys_by_node.insert(param.id, id);

                    parameters.push(self.rcx.ty(id).clone());
                }

                let return_ty_id = self.tr_resolve_ty(&extern_fun.return_ty);
                let kind = TyKind::Fn(parameters, Box::new(self.rcx.ty(return_ty_id).clone()));
                let id = self
                    .rcx
                    .ty_resolutions
                    .insert_with_key(|key| Ty { id: key, kind });

                self.rcx.tys_by_node.insert(item.id, id);
            }
        }
    }

    fn tr_walk_stmt(&mut self, stmt: &ast::Stmt) {
        match &stmt.kind {
            ast::StmtKind::Let(decl) => {
                self.tr_walk_expr(&decl.init);

                let ty = if let Some(ty) = &decl.ty {
                    self.tr_resolve_ty(ty)
                } else {
                    self.tr_infer_ty(stmt, decl)
                };

                self.tr_walk_pat_expr_pair(&decl.pat, &decl.init);

                self.rcx.tys_by_node.insert(stmt.id, ty);
            }
            ast::StmtKind::Loop(r#loop) => {
                for stmt in &r#loop.block.children {
                    self.tr_walk_stmt(stmt);
                }
            }
            ast::StmtKind::While(r#while) => {
                self.tr_walk_expr(&r#while.expr);

                for stmt in &r#while.block.children {
                    self.tr_walk_stmt(stmt);
                }
            }
            ast::StmtKind::Assignment(assign) => self.tr_walk_expr(&assign.value),
            ast::StmtKind::Expr(expr) => self.tr_walk_expr(expr),
            ast::StmtKind::If(r#if) => self.tr_walk_if_stmt(r#if),
            ast::StmtKind::Return(ret) => self.tr_walk_expr(&ret.expr),
            ast::StmtKind::Break => {}
        }
    }

    fn tr_walk_if_stmt(&mut self, r#if: &IfStmt) {
        self.tr_walk_expr(&r#if.expr);

        for stmt in &r#if.block.children {
            self.tr_walk_stmt(stmt);
        }

        match &r#if.alternate {
            Some(AlternateBranch::Another(stmt)) => self.tr_walk_if_stmt(stmt),
            Some(AlternateBranch::Final(block)) => {
                for stmt in &block.children {
                    self.tr_walk_stmt(stmt);
                }
            }
            None => {}
        }
    }

    fn tr_walk_pat_expr_pair(&mut self, pat: &ast::Pat, expr: &ast::Expr) {
        let expr_ty_id = self.tr_type_of_expr(expr);
        let expr_ty = self.rcx.ty(expr_ty_id);

        let mut collected = Vec::new();

        if let TyKind::Tuple(expr) = &expr_ty.kind
            && let PatKind::Tuple(pat) = &pat.kind
        {
            let mut queue =
                VecDeque::from_iter(expr.iter().enumerate().map(|(i, ty)| (ty, pat.get(i))));

            while let Some((ty, maybe_pat)) = queue.pop_front() {
                if let Some(pat) = maybe_pat {
                    collected.push((pat.id, ty.id));
                }

                if let TyKind::Tuple(nested_tys) = &ty.kind
                    && let Some(pat) = maybe_pat
                    && let PatKind::Tuple(nested_pats) = &pat.kind
                {
                    queue.extend(
                        nested_tys
                            .iter()
                            .enumerate()
                            .map(|(i, ty)| (ty, nested_pats.get(i))),
                    );
                }
            }
        }

        for (id, ty) in collected {
            self.rcx.tys_by_node.insert(id, ty);
        }

        self.rcx.tys_by_node.insert(pat.id, expr_ty_id);
    }

    fn tr_walk_expr(&mut self, expr: &ast::Expr) {
        let _ = self.tr_type_of_expr(expr);

        if let ExprKind::Tuple(elements) = &expr.kind {
            let mut queue = Vec::from_iter(elements);

            while let Some(expr) = queue.pop() {
                let _ = self.tr_type_of_expr(expr); // We do this so tuples get their types inserted to the rcx.tys_of_node.

                if let ExprKind::Tuple(elements) = &expr.kind {
                    queue.extend(elements);
                }
            }
        }
    }

    #[must_use = "inferred types are not automatically inserted into the ResolutionContext"]
    fn tr_infer_ty(&mut self, stmt: &ast::Stmt, decl: &ast::LetStmt) -> TyId {
        let inferred = self.tr_type_of_expr(&decl.init);

        if self.rcx.ty(inferred).is_error() {
            self.dcx.emit(
                Diagnostic::error(self.file_id, stmt.span, Recovery::Fatal)
                    .with_message("unable to infer type")
                    .with_label(Label::primary(
                        Span::from((stmt.span.start, decl.pat.span.end)),
                        "unable to infer type",
                    ))
                    .with_label(Label::help(
                        decl.init.span,
                        "initializer expression doesn't resolve to a clear type",
                    )),
            );
        }

        inferred
    }

    fn tr_type_of_expr(&mut self, expr: &ast::Expr) -> TyId {
        if !self.rcx.tys_by_node.contains_key(&expr.id) {
            let id = self.tr_evaluate_type_of_expr(expr);

            self.rcx.tys_by_node.insert(expr.id, id);

            return id;
        }

        self.rcx.tys_by_node[&expr.id]
    }

    fn tr_evaluate_type_of_expr(&mut self, expr: &ast::Expr) -> TyId {
        let kind = match &expr.kind {
            ExprKind::Tuple(inner) => TyKind::Tuple(
                inner
                    .iter()
                    .map(|child| {
                        let id = self.tr_evaluate_type_of_expr(child);

                        self.rcx.ty(id).clone()
                    })
                    .collect(),
            ),
            ExprKind::Lit(lit) => TyKind::Prim(match lit {
                Literal::Bool { .. } => PrimTy::Boolean,
                Literal::Integer { .. } => PrimTy::Integer,
                Literal::Str { .. } | Literal::InterpolatedStr { .. } => PrimTy::Str,
            }),
            ExprKind::Path(path) => {
                let target = self.rcx.get_name_res(path.id);

                match target {
                    Def::Def(id) => {
                        let item = self.sess.item(id);

                        match &item.kind {
                            ayuc_session::ItemKind::Fn { .. }
                            | ayuc_session::ItemKind::ExternFn { .. } => {
                                return self.rcx.tys_by_node[&item.id];
                            }
                            _ => TyKind::Error,
                        }
                    }
                    Def::Local(id) => {
                        let local = self.sess.local(id);

                        return self.rcx.tys_by_node[&local.id];
                    }
                    Def::Error => TyKind::Error,
                }
            }
            ExprKind::Call(call) => {
                let callee_ty = self.tr_type_of_expr(&call.callee);

                if let TyKind::Fn(_, ret) = &self.rcx.ty(callee_ty).kind {
                    return ret.id;
                } else {
                    TyKind::Error
                }
            }
            ExprKind::Parenthesized(expr) => {
                return self.tr_evaluate_type_of_expr(expr);
            }
            ExprKind::Binary(bin) => match bin.operator {
                Operator::Add
                | Operator::Div
                | Operator::Minus
                | Operator::Modulus
                | Operator::Mul => TyKind::Prim(PrimTy::Integer),
                Operator::EqualsEquals
                | Operator::Gt
                | Operator::GtOrEqual
                | Operator::Lt
                | Operator::LtOrEqual
                | Operator::NotEquals => TyKind::Prim(PrimTy::Boolean),
            },
        };

        self.rcx
            .ty_resolutions
            .insert_with_key(|id| Ty { id, kind })
    }

    #[must_use = "resolved types are not automatically inserted into the ResolutionContext"]
    fn tr_resolve_ty(&mut self, ty: &ast::Ty) -> TyId {
        let kind = match &ty.kind {
            ast::TyKind::Tuple(inner) => TyKind::Tuple(
                inner
                    .iter()
                    .map(|child| {
                        let id = self.tr_resolve_ty(child);

                        self.rcx.ty(id).clone() // can we optimise this?
                    })
                    .collect(),
            ),
            ast::TyKind::Path(p) => {
                if p.segments.len() == 1 {
                    let segment = &p.segments[0];

                    if let Some(prim) = PrimTy::from_name(segment.ident.sym) {
                        TyKind::Prim(prim)
                    } else {
                        TyKind::Error
                    }
                } else {
                    TyKind::Error // not yet.
                }
            }
        };

        if kind == TyKind::Error
            && let ast::TyKind::Path(p) = &ty.kind
        {
            self.dcx.emit(
                Diagnostic::error(self.file_id, ty.span, Recovery::Fatal)
                    .with_message(format!("cannot find type `{}` in this scope", p))
                    .with_label(Label::primary(ty.span, "not found in this scope")),
            );
        }

        self.rcx
            .ty_resolutions
            .insert_with_key(|key| Ty { id: key, kind })
    }
}
