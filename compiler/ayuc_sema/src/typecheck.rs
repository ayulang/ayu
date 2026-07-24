use std::collections::VecDeque;

use ayuc_ast::{
    AssignStmt, CallExpr, ExprKind, FnItem, IfStmt, Item, LetStmt, PatKind, ReturnStmt, Stmt,
    WhileStmt,
};
use ayuc_ast_visit::{visitor::Visitor, walkable::Walkable};
use ayuc_diagnostic::{Diagnostic, DiagnosticContext, Label, Recovery};
use ayuc_resolve::{PrimTy, Ty, TyKind, def::Def, resolver::ResolutionContext};
use ayuc_session::Session;
use ayuc_span::Span;

#[derive(Default)]
struct State<'rcx, 'ast> {
    current_item: Option<&'ast Item>,
    current_stmt: Option<&'ast Stmt>,

    return_ty: Option<&'rcx Ty>,
}

pub struct TypeCheckingPhase<'a, 'rcx, 'ast> {
    dcx: &'a mut DiagnosticContext,
    rcx: &'rcx ResolutionContext,
    sess: &'a Session,
    file_id: usize,

    state: State<'rcx, 'ast>,
}

impl<'a, 'rcx> TypeCheckingPhase<'a, 'rcx, '_> {
    pub fn new(
        dcx: &'a mut DiagnosticContext,
        rcx: &'rcx ResolutionContext,
        sess: &'a Session,
        file_id: usize,
    ) -> Self {
        Self {
            dcx,
            rcx,
            sess,
            file_id,
            state: State::default(),
        }
    }
}

impl<'ast> Visitor<'ast> for TypeCheckingPhase<'_, '_, 'ast> {
    fn visit_item(&mut self, item: &'ast Item) {
        let old_item = self.state.current_item.replace(item);

        item.walk(self);

        self.state.current_item = old_item;
    }

    fn visit_fn_item(&mut self, fun: &'ast FnItem) {
        let Some(item) = self.state.current_item else {
            return fun.walk(self);
        };

        let TyKind::Fn(_, return_ty) = &self.rcx.ty_of(item.id).kind else {
            unreachable!()
        };

        let old_ty = self.state.return_ty.replace(return_ty);

        fun.walk(self);

        self.state.return_ty = old_ty;
    }

    fn visit_return_stmt(&mut self, ret: &'ast ReturnStmt) {
        let Some(return_ty) = self.state.return_ty else {
            return ret.walk(self);
        };

        let Some(stmt) = self.state.current_stmt else {
            return ret.walk(self);
        };

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

        ret.walk(self)
    }

    fn visit_stmt(&mut self, stmt: &'ast Stmt) {
        let old_stmt = self.state.current_stmt.replace(stmt);

        stmt.walk(self);

        self.state.current_stmt = old_stmt;
    }

    fn visit_call_expression(&mut self, call: &'ast CallExpr) {
        let Some(stmt) = self.state.current_stmt else {
            return call.walk(self);
        };

        let TyKind::Fn(parameters, _) = &self.rcx.ty_of(call.callee.id).kind else {
            return call.walk(self);
        };

        let mut missing_args = Vec::new();
        let mut unexpected_args = Vec::new();
        let mut incorrect_args = Vec::new();

        for (i, provided) in call.args.iter().enumerate() {
            let provided_ty = self.rcx.ty_of(provided.id);

            if let Some(param_ty) = parameters.get(i) {
                if param_ty != provided_ty {
                    incorrect_args.push((provided.span, param_ty, provided_ty));
                }
            } else {
                unexpected_args.push((i, provided_ty, provided.span));
            }
        }

        if unexpected_args.is_empty() {
            for (i, param_ty) in parameters.iter().enumerate() {
                if call.args.get(i).is_none() {
                    missing_args.push((i, param_ty));
                }
            }
        }

        if !missing_args.is_empty() || !incorrect_args.is_empty() || !unexpected_args.is_empty() {
            let message = if !missing_args.is_empty() || !unexpected_args.is_empty() {
                &format!(
                    "function takes {} arguments, but {} arguments were supplied",
                    parameters.len(),
                    call.args.len()
                )
            } else {
                "arguments to this function are incorrect"
            };

            let mut diagnostic =
                Diagnostic::error(self.file_id, stmt.span, Recovery::Fatal).with_message(message);

            if !missing_args.is_empty() {
                if missing_args.len() == 1 {
                    let (position, ty) = missing_args.remove(0);

                    diagnostic = diagnostic.with_label(Label::primary(
                        Span::from((call.callee.span.end, stmt.span.end)),
                        format!("argument #{} of type {ty} is missing", position + 1),
                    ))
                } else {
                    let types = {
                        let first_part = &missing_args[0..missing_args.len() - 1];
                        let (_, last) = missing_args.last().unwrap(); // garuanteed to be there

                        format!(
                            "{} and `{last}`",
                            first_part
                                .iter()
                                .map(|(_, ty)| format!("`{ty}`"))
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    };

                    diagnostic = diagnostic.with_label(Label::primary(
                        Span::from((call.callee.span.end, stmt.span.end)),
                        format!(
                            "{} arguments of type {} are missing",
                            missing_args.len(),
                            types
                        ),
                    ))
                }
            }

            for (position, ty, span) in unexpected_args {
                diagnostic = diagnostic.with_label(Label::primary(
                    span,
                    format!("unexpected argument #{} of type `{ty}`", position + 1),
                ))
            }

            for (span, expected_ty, provided_ty) in incorrect_args {
                diagnostic = diagnostic.with_label(Label::primary(
                    span,
                    format!("expected `{expected_ty}`, found `{provided_ty}`"),
                ))
            }

            self.dcx.emit(diagnostic);
        }

        call.walk(self)
    }

    fn visit_while_stmt(&mut self, while_stmt: &'ast WhileStmt) {
        let condition_ty = self.rcx.ty_of(while_stmt.expr.id);

        if !matches!(condition_ty.kind, TyKind::Prim(PrimTy::Boolean)) {
            self.dcx.emit(
                Diagnostic::error(self.file_id, while_stmt.expr.span, Recovery::Fatal)
                    .with_message("condition of while statement must be of type bool")
                    .with_label(Label::primary(
                        while_stmt.expr.span,
                        format!("expected bool, got {condition_ty}"),
                    )),
            );
        }

        while_stmt.walk(self);
    }

    fn visit_if_stmt(&mut self, if_stmt: &'ast IfStmt) {
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

        if_stmt.walk(self);
    }

    fn visit_assign_stmt(&mut self, assign_stmt: &'ast AssignStmt) {
        let Def::Local(local) = self.rcx.get_name_res(assign_stmt.ident.id) else {
            return assign_stmt.walk(self);
        };

        let Some(stmt) = self.state.current_stmt else {
            return assign_stmt.walk(self);
        };

        let info = self.sess.local(local);

        let ty = self.rcx.ty_of(info.id);
        let expr_ty = self.rcx.ty_of(assign_stmt.value.id);

        if ty.is_error() || expr_ty.is_error() {
            return assign_stmt.walk(self);
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
                        assign_stmt.value.span,
                        format!("this is of type {}", expr_ty),
                    )),
            );
        }

        assign_stmt.walk(self)
    }

    fn visit_let_stmt(&mut self, let_stmt: &'ast LetStmt) {
        let Some(stmt) = self.state.current_stmt else {
            return let_stmt.walk(self);
        };

        let decl_ty = self.rcx.ty_of(stmt.id);
        let expr_ty = self.rcx.ty_of(let_stmt.init.id);

        if decl_ty.is_error() || expr_ty.is_error() {
            return let_stmt.walk(self);
        }

        if expr_ty != decl_ty {
            self.dcx.emit(
                Diagnostic::error(self.file_id, stmt.span, Recovery::Fatal)
                    .with_message(format!("expected type {}, got type {}", decl_ty, expr_ty))
                    .with_label(Label::help(
                        Span::from((
                            stmt.span.start,
                            match &let_stmt.ty {
                                Some(ty) => ty.span.end,
                                None => let_stmt.pat.span.end,
                            },
                        )),
                        format!("this is of type {}", decl_ty),
                    ))
                    .with_label(Label::primary(
                        let_stmt.init.span,
                        format!("this is of type {}", expr_ty),
                    )),
            );

            return let_stmt.walk(self);
        }

        // Pattern exhaustiveness check

        if let ExprKind::Tuple(expr_elements) = &let_stmt.init.kind
            && let PatKind::Tuple(pat_elements) = &let_stmt.pat.kind
        {
            fn exhaustiveness_check(
                this: &mut TypeCheckingPhase,
                matched: usize,
                existing: usize,
                pat_span: Span,
                tuple_span: Span,
                expr_ty: &Ty,
            ) {
                if matched != existing {
                    let mut diagnostic = Diagnostic::error(this.file_id, pat_span, Recovery::Fatal)
                        .with_message("mismatched types")
                        .with_label(Label::primary(
                            pat_span,
                            format!(
                                "expected a tuple with {} elements, found one with {} elements",
                                existing, matched
                            ),
                        ))
                        .with_label(Label::help(
                            tuple_span,
                            format!("this is of type `{expr_ty}`"),
                        ));

                    if existing > matched {
                        diagnostic = diagnostic.with_help(
                            "consider discarding the missing elements with a `_` variable",
                        );
                    }

                    this.dcx.emit(diagnostic);
                }
            }

            exhaustiveness_check(
                self,
                pat_elements.len(),
                expr_elements.len(),
                let_stmt.pat.span,
                let_stmt.init.span,
                expr_ty,
            );

            let mut queue = VecDeque::from_iter(
                expr_elements
                    .iter()
                    .enumerate()
                    .map(|(i, expr)| (expr, pat_elements.get(i))),
            );

            while let Some((expr, maybe_pat)) = queue.pop_front() {
                if let ExprKind::Tuple(nested_tys) = &expr.kind
                    && let Some(pat) = maybe_pat
                    && let PatKind::Tuple(nested_pats) = &pat.kind
                {
                    queue.extend(
                        nested_tys
                            .iter()
                            .enumerate()
                            .map(|(i, ty)| (ty, nested_pats.get(i))),
                    );

                    exhaustiveness_check(
                        self,
                        nested_tys.len(),
                        nested_pats.len(),
                        pat.span,
                        expr.span,
                        self.rcx.ty_of(expr.id),
                    );
                }
            }
        }

        let_stmt.walk(self)
    }
}
