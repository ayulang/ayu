use crate::{
    def::Def,
    resolver::Resolver,
    ty::{PrimTy, Ty},
};

use ayuc_ast::{self as ast, AlternateBranch, ExprKind, IfStmt, Literal, Operator};
use ayuc_diagnostic::{Diagnostic, Label, Recovery};
use ayuc_span::Span;

impl Resolver<'_, '_> {
    pub(crate) fn resolve_types(&mut self, ast: &ast::Ast) {
        for item in &ast.items {
            self.tr_walk_item(item);
        }
    }

    fn tr_walk_item(&mut self, item: &ast::Item) {
        match &item.kind {
            ast::ItemKind::InlineMod(ast::InlineModItem { items, .. })
            | ast::ItemKind::ExternMod(ast::ExternModItem { items, .. }) => {
                for item in items {
                    self.tr_walk_item(item);
                }
            }
            ast::ItemKind::Fn(fun) => {
                let mut parameters = Vec::with_capacity(fun.parameters.parameters.len());

                for param in &fun.parameters.parameters {
                    let ty = self.tr_resolve_ty(&param.ty);

                    self.rcx.ty_resolutions.insert(param.id, ty.clone());
                    parameters.push(ty);
                }

                let return_ty = self.tr_resolve_ty(&fun.return_ty);

                self.rcx
                    .ty_resolutions
                    .insert(item.id, Ty::Fn(parameters, Box::new(return_ty)));

                for stmt in &fun.block.children {
                    self.tr_walk_stmt(stmt);
                }
            }
            ast::ItemKind::ExternFn(extern_fun) => {
                let mut parameters = Vec::with_capacity(extern_fun.parameters.parameters.len());

                for param in &extern_fun.parameters.parameters {
                    let ty = self.tr_resolve_ty(&param.ty);

                    self.rcx.ty_resolutions.insert(param.id, ty.clone());
                    parameters.push(ty);
                }

                let return_ty = self.tr_resolve_ty(&extern_fun.return_ty);

                self.rcx
                    .ty_resolutions
                    .insert(item.id, Ty::Fn(parameters, Box::new(return_ty)));
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

                self.rcx.ty_resolutions.insert(stmt.id, ty);
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

    fn tr_walk_expr(&mut self, expr: &ast::Expr) {
        let _ = self.tr_type_of_expr(expr);
    }

    #[must_use = "inferred types are not automatically inserted into the ResolutionContext"]
    fn tr_infer_ty(&mut self, stmt: &ast::Stmt, decl: &ast::LetStmt) -> Ty {
        let inferred = self.tr_type_of_expr(&decl.init).clone();

        if inferred.is_error() {
            self.dcx.emit(
                Diagnostic::error(self.file_id, stmt.span, Recovery::Fatal)
                    .with_message(format!("unable to infer type of `{}`", decl.ident.sym))
                    .with_label(Label::primary(
                        Span::from((stmt.span.start, decl.ident.span.end)),
                        "unable to infer type",
                    ))
                    .with_label(Label::help(
                        decl.init.span,
                        "initializer expression doesn't resolve to a clear type",
                    ))
                    .with_help(format!("consider assigning a type to `{}`", decl.ident.sym)),
            );
        }

        inferred
    }

    fn tr_type_of_expr(&mut self, expr: &ast::Expr) -> &Ty {
        if !self.rcx.ty_resolutions.contains_key(&expr.id) {
            let evaluated = self.tr_evaluate_type_of_expr(expr);

            self.rcx.ty_resolutions.insert(expr.id, evaluated);
        }

        &self.rcx.ty_resolutions[&expr.id]
    }

    fn tr_evaluate_type_of_expr(&mut self, expr: &ast::Expr) -> Ty {
        match &expr.kind {
            ExprKind::Tuple(inner) => Ty::Tuple(
                inner
                    .iter()
                    .map(|child| self.tr_evaluate_type_of_expr(child))
                    .collect(),
            ),
            ExprKind::Lit(lit) => Ty::Prim(match lit {
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
                                self.rcx.ty_res(item.id).clone()
                            }
                            _ => Ty::Error,
                        }
                    }
                    Def::Local(id) => {
                        let local = self.sess.local(id);

                        self.rcx.ty_res(local.id).clone()
                    }
                    Def::Error => Ty::Error,
                }
            }
            ExprKind::Call(call) => {
                let callee_ty = self.tr_type_of_expr(&call.callee);

                if let Ty::Fn(_, ret) = callee_ty {
                    ret.as_ref().clone()
                } else {
                    Ty::Error
                }
            }
            ExprKind::Parenthesized(expr) => self.tr_evaluate_type_of_expr(expr),
            ExprKind::Binary(bin) => match bin.operator {
                Operator::Add
                | Operator::Div
                | Operator::Minus
                | Operator::Modulus
                | Operator::Mul => Ty::Prim(PrimTy::Integer),
                Operator::EqualsEquals
                | Operator::Gt
                | Operator::GtOrEqual
                | Operator::Lt
                | Operator::LtOrEqual
                | Operator::NotEquals => Ty::Prim(PrimTy::Boolean),
            },
        }
    }

    #[must_use = "resolved types are not automatically inserted into the ResolutionContext"]
    fn tr_resolve_ty(&mut self, ty: &ast::Ty) -> Ty {
        let resolved = match &ty.kind {
            ast::TyKind::Tuple(inner) => Ty::Tuple(
                inner
                    .iter()
                    .map(|child| self.tr_resolve_ty(child))
                    .collect(),
            ),
            ast::TyKind::Path(p) => {
                if p.segments.len() == 1 {
                    let segment = &p.segments[0];

                    if let Some(prim) = PrimTy::from_name(segment.ident.sym) {
                        Ty::Prim(prim)
                    } else {
                        Ty::Error
                    }
                } else {
                    Ty::Error // not yet.
                }
            }
        };

        if resolved == Ty::Error
            && let ast::TyKind::Path(p) = &ty.kind
        {
            self.dcx.emit(
                Diagnostic::error(self.file_id, ty.span, Recovery::Fatal)
                    .with_message(format!("cannot find type `{}` in this scope", p))
                    .with_label(Label::primary(ty.span, "not found in this scope")),
            );
        }

        resolved
    }
}
