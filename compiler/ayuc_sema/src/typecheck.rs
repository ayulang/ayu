use std::collections::VecDeque;

use ayuc_ast::{
    self as ast, AlternateBranch, FnDecl, IfStmt, Item, ItemKind, LoopStmt, PatKind, StmtKind,
    WhileStmt,
};
use ayuc_diagnostic::{Diagnostic, Label, Recovery};
use ayuc_id::ast::NodeId;
use ayuc_resolve::{PrimTy, Ty, TyKind, def::Def};
use ayuc_span::Span;

use crate::SemanticAnalyzer;

impl SemanticAnalyzer<'_> {
    pub fn typecheck(&mut self, ast: &ast::Ast) {
        for item in &ast.items {
            self.tc_walk_item(item);
        }
    }

    fn tc_walk_item(&mut self, item: &Item) {
        match &item.kind {
            ItemKind::Fn(decl) => self.tc_walk_fn_item(item.id, decl),
            ItemKind::InlineMod(decl) => {
                for member in &decl.items {
                    self.tc_walk_item(member);
                }
            }
            ItemKind::ExternMod(decl) => {
                for member in &decl.items {
                    self.tc_walk_item(member);
                }
            }
            ItemKind::ExternFn(_) => {}
        }
    }

    fn tc_walk_fn_item(&mut self, item_id: NodeId, item: &FnDecl) {
        let TyKind::Fn(_, return_ty) = &self.rcx.ty_of(item_id).kind else {
            unreachable!()
        };

        for stmt in &item.block.children {
            self.tc_walk_stmt(stmt);
            self.tc_check_for_return(stmt, return_ty);
        }
    }

    fn tc_check_for_return(&mut self, stmt: &ast::Stmt, return_ty: &Ty) {
        match &stmt.kind {
            StmtKind::Return(ret) => {
                let ty = self.rcx.ty_of(ret.expr.id);

                if ty != return_ty {
                    self.dcx.emit(
                        Diagnostic::error(self.file_id, stmt.span, Recovery::Fatal)
                            .with_message("incorrect return type")
                            .with_label(Label::primary(
                                if self.sess.is_synthetic(ret.expr.id) {
                                    stmt.span
                                } else {
                                    ret.expr.span
                                },
                                format!("expected type {}, got {}", return_ty, ty,),
                            )),
                    );
                }
            }
            StmtKind::If(if_stmt) => self.tc_check_if_for_return(if_stmt, return_ty),
            StmtKind::Loop(LoopStmt { block }) | StmtKind::While(WhileStmt { block, .. }) => {
                for stmt in &block.children {
                    self.tc_check_for_return(stmt, return_ty);
                }
            }
            StmtKind::Assignment(_) | StmtKind::Let(_) | StmtKind::Expr(_) | StmtKind::Break => {}
        }
    }

    fn tc_check_if_for_return(&mut self, if_stmt: &IfStmt, return_ty: &Ty) {
        for stmt in &if_stmt.block.children {
            self.tc_check_for_return(stmt, return_ty);
        }

        match &if_stmt.alternate {
            Some(AlternateBranch::Final(block)) => {
                for stmt in &block.children {
                    self.tc_check_for_return(stmt, return_ty);
                }
            }
            Some(AlternateBranch::Another(if_stmt)) => {
                self.tc_check_if_for_return(if_stmt, return_ty)
            }
            None => {}
        }
    }

    fn tc_walk_stmt(&mut self, stmt: &ast::Stmt) {
        match &stmt.kind {
            ast::StmtKind::Loop(r#loop) => {
                for stmt in &r#loop.block.children {
                    self.tc_walk_stmt(stmt);
                }
            }
            ast::StmtKind::While(r#while) => {
                let condition_ty = self.rcx.ty_of(r#while.expr.id);

                if !matches!(condition_ty.kind, TyKind::Prim(PrimTy::Boolean)) {
                    self.dcx.emit(
                        Diagnostic::error(self.file_id, r#while.expr.span, Recovery::Fatal)
                            .with_message("condition of while statement must be of type bool")
                            .with_label(Label::primary(
                                r#while.expr.span,
                                format!("expected bool, got {condition_ty}"),
                            )),
                    );
                }

                for stmt in &r#while.block.children {
                    self.tc_walk_stmt(stmt);
                }
            }
            ast::StmtKind::Assignment(assign) => self.tc_check_assign_stmt(stmt, assign),
            ast::StmtKind::Let(decl) => self.tc_check_let_stmt(stmt, decl),
            ast::StmtKind::If(r#if) => self.tc_walk_if_stmt(r#if),
            ast::StmtKind::Expr(_) | ast::StmtKind::Return(_) | ast::StmtKind::Break => {}
        }
    }

    fn tc_walk_if_stmt(&mut self, if_stmt: &IfStmt) {
        let condition_ty = self.rcx.ty_of(if_stmt.expr.id);

        if !matches!(condition_ty.kind, TyKind::Prim(PrimTy::Boolean)) {
            self.dcx.emit(
                Diagnostic::error(self.file_id, if_stmt.expr.span, Recovery::Fatal)
                    .with_message("condition of if statements must be of type bool")
                    .with_label(Label::primary(
                        if_stmt.expr.span,
                        format!("expected bool, got {condition_ty}"),
                    )),
            );
        }

        for stmt in &if_stmt.block.children {
            self.tc_walk_stmt(stmt);
        }

        match &if_stmt.alternate {
            Some(AlternateBranch::Another(stmt)) => self.tc_walk_if_stmt(stmt),
            Some(AlternateBranch::Final(block)) => {
                for stmt in &block.children {
                    self.tc_walk_stmt(stmt);
                }
            }
            None => {}
        }
    }

    fn tc_check_assign_stmt(&mut self, stmt: &ast::Stmt, assign: &ast::AssignStmt) {
        let local = match self.rcx.get_name_res(assign.ident.id) {
            Def::Local(local) => local,
            _ => return,
        };

        let info = self.sess.local(local);

        let ty = self.rcx.ty_of(info.id);
        let expr_ty = self.rcx.ty_of(assign.value.id);

        if ty.is_error() || expr_ty.is_error() {
            return;
        }

        if ty != expr_ty {
            self.dcx.emit(
                Diagnostic::error(self.file_id, stmt.span, Recovery::Fatal)
                    .with_message(format!("expected type {}, got type {}", ty, expr_ty))
                    .with_label(Label::help(
                        info.defined_where,
                        format!("this is of type {}", ty),
                    ))
                    .with_label(Label::primary(
                        assign.value.span,
                        format!("this is of type {}", expr_ty),
                    )),
            );
        }
    }

    fn tc_check_let_stmt(&mut self, stmt: &ast::Stmt, decl: &ast::LetStmt) {
        let decl_ty = self.rcx.ty_of(stmt.id);
        let expr_ty = self.rcx.ty_of(decl.init.id);

        if decl_ty.is_error() || expr_ty.is_error() {
            return;
        }

        if expr_ty != decl_ty {
            self.dcx.emit(
                Diagnostic::error(self.file_id, stmt.span, Recovery::Fatal)
                    .with_message(format!("expected type {}, got type {}", decl_ty, expr_ty))
                    .with_label(Label::help(
                        Span::from((
                            stmt.span.start,
                            match &decl.ty {
                                Some(ty) => ty.span.end,
                                None => decl.pat.span.end,
                            },
                        )),
                        format!("this is of type {}", decl_ty),
                    ))
                    .with_label(Label::primary(
                        decl.init.span,
                        format!("this is of type {}", expr_ty),
                    )),
            );

            return;
        }

        // Pattern exhaustiveness check

        if let TyKind::Tuple(expr) = &expr_ty.kind
            && let PatKind::Tuple(pat) = &decl.pat.kind
        {
            let mut queue =
                VecDeque::from_iter(expr.iter().enumerate().map(|(i, ty)| (ty, pat.get(i))));

            while let Some((ty, maybe_pat)) = queue.pop_front() {
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

                    if nested_tys.len() != nested_pats.len() {
                        let mut diagnostic = Diagnostic::error(
                            self.file_id,
                            pat.span,
                            Recovery::Fatal,
                        )
                        .with_message("mismatched types")
                        .with_label(Label::primary(
                            pat.span,
                            format!(
                                "expected a tuple with {} elements, found one with {} elements",
                                nested_tys.len(),
                                nested_pats.len()
                            ),
                        ))
                        .with_label(Label::help(
                            decl.init.span,
                            format!("this expression has type `{expr_ty}`"),
                        ));

                        if nested_tys.len() > nested_pats.len() {
                            diagnostic = diagnostic.with_help(
                                "consider discarding the missing elements with a `_` variable",
                            );
                        }

                        self.dcx.emit(diagnostic);
                    }
                }
            }
        }
    }
}
