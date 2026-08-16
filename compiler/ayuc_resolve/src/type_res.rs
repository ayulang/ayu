use std::collections::VecDeque;

use ayuc_ast::{
    Ast, Expr, ExprKind, ExternFnItem, FnItem, Item, LetStmt, Literal, Operator, Parameter, Pat,
    PatKind, Stmt,
};
use ayuc_ast_visit::{visitor::Visitor, walkable::Walkable};
use ayuc_diagnostic::{Diagnostic, Label, Recovery};
use ayuc_id::TyId;
use ayuc_span::Span;
use ayuc_type::ty::{PrimTy, TyKind};

use crate::{Def, Resolver};

impl<'dcx, 'sess> Resolver<'dcx, 'sess, '_> {
    pub(crate) fn run_type_resolution(&mut self, ast: &Ast) {
        TypeResolutionPhase {
            res: self,
            current_item: None,
            current_stmt: None,
            current_ty: None,
        }
        .visit_ast(ast);
    }
}

pub struct TypeResolutionPhase<'a, 'dcx, 'sess, 'ast, 'reg> {
    res: &'a mut Resolver<'dcx, 'sess, 'reg>,

    current_item: Option<&'ast Item>,
    current_stmt: Option<&'ast Stmt>,
    current_ty: Option<&'ast ayuc_ast::Ty>,
}

impl TypeResolutionPhase<'_, '_, '_, '_, '_> {
    fn evaluate_type_of_expr(&mut self, expr: &Expr) -> TyId {
        let kind = match &expr.kind {
            ExprKind::Tuple(inner) => TyKind::Tuple(
                inner
                    .iter()
                    .map(|child| self.evaluate_type_of_expr(child))
                    .collect(),
            ),
            ExprKind::Lit(lit) => TyKind::Prim(match lit {
                Literal::Bool { .. } => PrimTy::Bool,
                Literal::Integer { .. } => PrimTy::Int,
                Literal::Str { .. } | Literal::InterpolatedStr { .. } => PrimTy::Str,
            }),
            ExprKind::Path(path) => {
                let target = self.res.rcx.get_name_res(path.id);

                match target {
                    Def::Def(id) => {
                        // For now, only function items can have a type.
                        if let Some(id) = self.res.sess.items[id].fn_ty_id() {
                            return id;
                        } else {
                            TyKind::Error
                        }
                    }
                    Def::Local(id) => {
                        let local = &self.res.sess.locals[id];

                        return self.res.rcx.ty_id_of(local.id);
                    }
                    Def::Error => TyKind::Error,
                }
            }
            ExprKind::Call(call) => {
                let callee_ty = self.res.ty_of(call.callee.id);

                if let TyKind::Fn(_, ret) = callee_ty {
                    return *ret;
                } else {
                    TyKind::Error
                }
            }
            ExprKind::Parenthesized(expr) => {
                return self.evaluate_type_of_expr(expr);
            }
            ExprKind::Binary(bin) => match bin.operator {
                Operator::Add
                | Operator::Div
                | Operator::Minus
                | Operator::Modulus
                | Operator::Mul => TyKind::Prim(PrimTy::Int),
                Operator::EqualsEquals
                | Operator::Gt
                | Operator::GtOrEqual
                | Operator::Lt
                | Operator::LtOrEqual
                | Operator::NotEquals => TyKind::Prim(PrimTy::Bool),
            },
        };

        self.res.sess.interner.intern(kind)
    }

    fn tr_walk_pat_expr_pair(&mut self, pat: &Pat, expr: &Expr) {
        let expr_ty_id = self.res.rcx.ty_id_of(expr.id);
        let mut collected = Vec::new();

        {
            let expr_ty = self.res.ty(expr_ty_id);

            if let TyKind::Tuple(expr) = &expr_ty
                && let PatKind::Tuple(pat) = &pat.kind
            {
                let mut queue = VecDeque::from_iter(
                    expr.iter()
                        .enumerate()
                        .map(|(i, ty_id)| (*ty_id, pat.get(i))),
                );

                while let Some((ty_id, maybe_pat)) = queue.pop_front() {
                    if let Some(pat) = maybe_pat {
                        collected.push((pat.id, ty_id));
                    }

                    if let TyKind::Tuple(nested_tys) = &self.res.ty(ty_id)
                        && let Some(pat) = maybe_pat
                        && let PatKind::Tuple(nested_pats) = &pat.kind
                    {
                        queue.extend(
                            nested_tys
                                .iter()
                                .enumerate()
                                .map(|(i, ty_id)| (*ty_id, nested_pats.get(i))),
                        );
                    }
                }
            }
        }

        for (id, ty) in collected {
            self.res.rcx.tys_by_node.insert(id, ty);
        }

        self.res.rcx.tys_by_node.insert(pat.id, expr_ty_id);
    }
}

impl<'ast> Visitor<'ast> for TypeResolutionPhase<'_, '_, '_, 'ast, '_> {
    fn visit_item(&mut self, item: &'ast Item) {
        let old_item = self.current_item.replace(item);

        item.walk(self);

        self.current_item = old_item;
    }

    fn visit_parameter(&mut self, parameter: &'ast Parameter) {
        parameter.walk(self);

        self.res
            .rcx
            .tys_by_node
            .insert(parameter.id, self.res.rcx.tys_by_node[&parameter.ty.id]);
    }

    fn visit_fn_item(&mut self, fun: &'ast FnItem) {
        let item = self
            .current_item
            .expect("visit_fn_item called outside of item context");

        let def_id = self.res.rcx.defs_by_node[&item.id];

        fun.walk(self); // So the `Ty`s can be resolved

        let sess_item = &mut self.res.sess.items[def_id];
        let ayuc_item::ItemKind::Fn(fn_item) = &mut sess_item.kind else {
            unreachable!()
        };

        let mut parameters = Vec::with_capacity(fun.parameters.parameters.len());

        for (i, param) in fun.parameters.parameters.iter().enumerate() {
            let ty_id = self.res.rcx.ty_id_of(param.ty.id);

            parameters.push(ty_id);
            fn_item.parameters[i].ty_id = Some(ty_id);
        }

        let return_ty_id = self.res.rcx.ty_id_of(fun.return_ty.id);
        let kind = TyKind::Fn(parameters, return_ty_id);
        let id = self.res.sess.interner.intern(kind);

        fn_item.ty_id = Some(id);

        self.res.rcx.tys_by_node.insert(item.id, id);
    }

    fn visit_extern_fn_item(&mut self, extern_fun: &'ast ExternFnItem) {
        let item = self
            .current_item
            .expect("visit_fn_item called outside of item context");

        let def_id = self.res.rcx.defs_by_node[&item.id];

        extern_fun.walk(self); // So the `Ty`s can be resolved

        let sess_item = &mut self.res.sess.items[def_id];
        let ayuc_item::ItemKind::ExternFn(fn_item) = &mut sess_item.kind else {
            unreachable!()
        };

        let mut parameters = Vec::with_capacity(extern_fun.parameters.parameters.len());

        for (i, param) in extern_fun.parameters.parameters.iter().enumerate() {
            let ty_id = self.res.rcx.ty_id_of(param.ty.id);

            parameters.push(ty_id);
            fn_item.parameters[i].ty_id = Some(ty_id);
        }

        let return_ty_id = self.res.rcx.ty_id_of(extern_fun.return_ty.id);
        let kind = TyKind::Fn(parameters, return_ty_id);
        let id = self.res.sess.interner.intern(kind);

        fn_item.ty_id = Some(id);

        self.res.rcx.tys_by_node.insert(item.id, id);
    }

    fn visit_path_ty(&mut self, path: &'ast ayuc_ast::Path) {
        let current_ty = self
            .current_ty
            .expect("visit_tuple_ty called outside of type context");

        let kind = if path.segments.len() == 1 {
            let segment = &path.segments[0];

            if let Some(prim) = PrimTy::from_name(segment.ident.sym.as_str()) {
                TyKind::Prim(prim)
            } else {
                TyKind::Error
            }
        } else {
            TyKind::Error // not yet.
        };

        let id = self.res.sess.interner.intern(kind);

        self.res.rcx.tys_by_node.insert(current_ty.id, id);
    }

    fn visit_tuple_ty(&mut self, elements: &'ast [ayuc_ast::Ty]) {
        let current_ty = self
            .current_ty
            .expect("visit_tuple_ty called outside of type context");

        elements.walk(self); // runs `visit_ty` on all elements

        let kind = TyKind::Tuple(
            elements
                .iter()
                .map(|child| self.res.rcx.ty_id_of(child.id))
                .collect(),
        );

        let id = self.res.sess.interner.intern(kind);

        self.res.rcx.tys_by_node.insert(current_ty.id, id);
    }

    fn visit_ty(&mut self, ty: &'ast ayuc_ast::Ty) {
        let old_ty = self.current_ty.replace(ty);

        ty.walk(self);

        self.current_ty = old_ty;

        let resolved_ty_id = self.res.rcx.ty_id_of(ty.id);
        let resolved_ty = self.res.ty(resolved_ty_id);

        if resolved_ty.is_error()
            && let ayuc_ast::TyKind::Path(p) = &ty.kind
        {
            self.res.dcx.emit(
                Diagnostic::error(self.res.file_id, ty.span, Recovery::Fatal)
                    .with_message(format!("cannot find type `{}` in this scope", p))
                    .with_label(Label::primary(ty.span, "not found in this scope")),
            );
        }
    }

    fn visit_expr(&mut self, expr: &'ast ayuc_ast::Expr) {
        expr.walk(self); // resolve all children types first

        if !self.res.rcx.tys_by_node.contains_key(&expr.id) {
            let id = self.evaluate_type_of_expr(expr);

            self.res.rcx.tys_by_node.insert(expr.id, id);
        }
    }

    fn visit_stmt(&mut self, stmt: &'ast Stmt) {
        let old_stmt = self.current_stmt.replace(stmt);

        stmt.walk(self);

        self.current_stmt = old_stmt;
    }

    fn visit_let_stmt(&mut self, let_stmt: &'ast LetStmt) {
        let stmt = self
            .current_stmt
            .expect("visit_let_stmt called outside of statement context");

        let_stmt.walk(self);

        let ty = if let Some(ty) = &let_stmt.ty {
            self.res.rcx.ty_id_of(ty.id)
        } else {
            let inferred_id = self.res.rcx.ty_id_of(let_stmt.init.id);
            let inferred = self.res.ty(inferred_id);

            if inferred.is_error() {
                self.res.dcx.emit(
                    Diagnostic::error(self.res.file_id, stmt.span, Recovery::Fatal)
                        .with_message("unable to infer type")
                        .with_label(Label::primary(
                            Span::from((stmt.span.start, let_stmt.pat.span.end)),
                            "unable to infer type",
                        ))
                        .with_label(Label::help(
                            let_stmt.init.span,
                            "initializer expression doesn't resolve to a clear type",
                        )),
                );
            }

            inferred_id
        };

        self.res.rcx.tys_by_node.insert(stmt.id, ty);

        self.tr_walk_pat_expr_pair(&let_stmt.pat, &let_stmt.init);
    }
}
