use ayuc_ast::{self as ast};
use ayuc_hir::{self as hir};

use ayuc_id::{
    ast::NodeId,
    hir::{DefId, HirId, HirIdAllocator},
};
use ayuc_resolve::{
    def::Def as RDef,
    resolver::ResolutionContext,
    ty::{PrimTy as RPrimTy, Ty as RTy, TyKind as RTyKind},
};
use bimap::BiHashMap;
use slotmap::SecondaryMap;

#[derive(Default)]
pub struct LoweringContext {
    pub items: SecondaryMap<DefId, hir::Item>,

    pub top_level_items: Vec<DefId>,
    pub id_mappings: BiHashMap<NodeId, HirId>,
}

pub struct AstLowering<'a> {
    ctx: LoweringContext,
    rcx: &'a ResolutionContext,

    hir_id_allocator: HirIdAllocator,
}

impl LoweringContext {
    #[inline]
    pub fn top_items(&self) -> Vec<(DefId, &hir::Item)> {
        self.top_level_items
            .iter()
            .map(|id| (*id, &self.items[*id]))
            .collect()
    }
}

impl<'a> AstLowering<'a> {
    pub fn new(rcx: &'a ResolutionContext) -> Self {
        Self {
            ctx: LoweringContext::default(),
            rcx,
            hir_id_allocator: HirIdAllocator::new(),
        }
    }

    #[must_use]
    pub fn lower(mut self, ast: &ayuc_ast::Ast) -> LoweringContext {
        for item in &ast.items {
            let def_id = self.rcx.defs_by_node[&item.id];
            let lowered = self.lower_item(item);

            self.ctx.items.insert(def_id, lowered);
            self.ctx.top_level_items.push(def_id);
        }

        self.ctx
    }

    #[must_use]
    fn lower_id(&mut self, id: NodeId) -> HirId {
        if self.ctx.id_mappings.get_by_left(&id).is_some() {
            panic!("tried to lower NodeId ({id:?}) into HirId: it has already been lowered");
        }

        let hir_id = self.hir_id_allocator.allocate();

        self.ctx.id_mappings.insert(id, hir_id);

        hir_id
    }

    fn lower_fn_item(&mut self, item: &ast::Item, fun: &ast::FnItem) -> hir::FnItem {
        let RTyKind::Fn(parameters, return_ty) = &self.rcx.ty_of(item.id).kind else {
            unreachable!()
        };

        let name = fun.ident.sym;
        let params = fun
            .parameters
            .parameters
            .iter()
            .enumerate()
            .map(|(i, p)| hir::Parameter {
                hir_id: self.hir_id_allocator.allocate(),
                name: p.ident.sym,
                ty: self.lower_res(&parameters[i]),
            })
            .collect::<Vec<_>>();

        let return_ty = self.lower_res(return_ty);
        let block = self.lower_block(&fun.block);

        hir::FnItem {
            name,
            block,
            params,
            return_ty,
        }
    }

    fn lower_item(&mut self, item: &ast::Item) -> hir::Item {
        let vis = match item.vis {
            ast::Visibility::Private => hir::Visibility::Private,
            ast::Visibility::Public => hir::Visibility::Public,
        };

        let id = self.rcx.defs_by_node[&item.id];
        let hir_id = self.lower_id(item.id);

        let kind = match &item.kind {
            ast::ItemKind::ExternMod(decl) => hir::ItemKind::ExternMod(hir::ExternModItem {
                name: decl.ident.sym,
                ffi_name: decl.ffi_name.as_ref().map(|i| i.sym),
                items: decl
                    .items
                    .iter()
                    .flat_map(|item| {
                        if matches!(item.kind, ast::ItemKind::Fn(_)) {
                            return None;
                        }

                        let def_id = self.rcx.defs_by_node[&item.id];
                        let lowered = self.lower_item(item);

                        self.ctx.items.insert(def_id, lowered);

                        Some(def_id)
                    })
                    .collect(),
            }),
            ast::ItemKind::InlineMod(decl) => hir::ItemKind::InlineMod(hir::InlineModItem {
                name: decl.ident.sym,
                items: decl
                    .items
                    .iter()
                    .map(|item| {
                        let def_id = self.rcx.defs_by_node[&item.id];
                        let lowered = self.lower_item(item);

                        self.ctx.items.insert(def_id, lowered);

                        def_id
                    })
                    .collect(),
            }),
            ast::ItemKind::Fn(fun) => hir::ItemKind::Fn(self.lower_fn_item(item, fun)),
            ast::ItemKind::ExternFn(extern_fun) => {
                let RTyKind::Fn(parameters, return_ty) = &self.rcx.ty_of(item.id).kind else {
                    unreachable!()
                };

                hir::ItemKind::ExternFn(hir::ExternFnItem {
                    name: extern_fun.name.sym,
                    ffi_name: extern_fun.ffi_name.as_ref().map(|i| i.sym),
                    return_ty: self.lower_res(return_ty),
                    params: extern_fun
                        .parameters
                        .parameters
                        .iter()
                        .enumerate()
                        .map(|(i, p)| hir::Parameter {
                            hir_id: self.hir_id_allocator.allocate(),
                            name: p.ident.sym,
                            ty: self.lower_res(&parameters[i]),
                        })
                        .collect(),
                })
            }
        };

        hir::Item {
            vis,
            id,
            hir_id,
            kind,
        }
    }

    fn lower_block(&mut self, block: &ast::Block) -> hir::Block {
        hir::Block {
            stmts: block.children.iter().map(|s| self.lower_stmt(s)).collect(),
        }
    }

    fn lower_pat(&mut self, pat: &ast::Pat) -> hir::Pat {
        hir::Pat {
            id: self.lower_id(pat.id),
            kind: match &pat.kind {
                ast::PatKind::Binding(binding) => hir::PatKind::Binding(hir::PatBinding {
                    sym: binding.sym,
                    mutable: binding.mutable,
                }),
                ast::PatKind::Tuple(parts) => {
                    hir::PatKind::Tuple(parts.iter().map(|part| self.lower_pat(part)).collect())
                }
            },
        }
    }

    fn lower_stmt(&mut self, stmt: &ast::Stmt) -> hir::Stmt {
        let id = self.lower_id(stmt.id);
        let kind = match &stmt.kind {
            ast::StmtKind::Break => hir::StmtKind::Break,
            ast::StmtKind::While(r#while) => hir::StmtKind::While(hir::WhileStmt {
                expr: self.lower_expr(&r#while.expr),
                block: self.lower_block(&r#while.block),
            }),
            ast::StmtKind::Loop(r#loop) => hir::StmtKind::Loop(hir::LoopStmt {
                block: self.lower_block(&r#loop.block),
            }),
            ast::StmtKind::Assignment(assign) => hir::StmtKind::Assign(hir::AssignStmt {
                ident: self.resolve_ident(&assign.ident),
                op: match assign.operator {
                    ast::AssignOperator::Add => hir::AssignOp::Add,
                    ast::AssignOperator::Assign => hir::AssignOp::Assign,
                    ast::AssignOperator::Subtract => hir::AssignOp::Sub,
                    ast::AssignOperator::Div => hir::AssignOp::Div,
                    ast::AssignOperator::Modulus => hir::AssignOp::Modulus,
                    ast::AssignOperator::Mul => hir::AssignOp::Mul,
                },
                value: self.lower_expr(&assign.value),
            }),
            ast::StmtKind::Expr(expr) => hir::StmtKind::Expr(self.lower_expr(expr)),
            ast::StmtKind::Let(decl) => hir::StmtKind::Let(hir::LetStmt {
                pat: self.lower_pat(&decl.pat),
                ty: self.lower_res(self.rcx.ty_of(stmt.id)),
                init: self.lower_expr(&decl.init),
            }),
            ast::StmtKind::Return(ret) => hir::StmtKind::Return(hir::ReturnStmt {
                expr: self.lower_expr(&ret.expr),
            }),
            ayuc_ast::StmtKind::If(if_stmt) => hir::StmtKind::If(self.lower_if_stmt(if_stmt)),
        };

        hir::Stmt { id, kind }
    }

    fn lower_if_stmt(&mut self, if_stmt: &ast::IfStmt) -> hir::IfStmt {
        hir::IfStmt {
            expr: self.lower_expr(&if_stmt.expr),
            block: self.lower_block(&if_stmt.block),
            alternate: if_stmt.alternate.as_ref().map(|alternate| match alternate {
                ast::AlternateBranch::Another(if_stmt) => {
                    hir::AlternateBranch::Another(Box::new(self.lower_if_stmt(if_stmt)))
                }
                ast::AlternateBranch::Final(block) => {
                    hir::AlternateBranch::Final(self.lower_block(block))
                }
            }),
        }
    }

    fn lower_expr(&mut self, expr: &ast::Expr) -> hir::Expr {
        let id = self.lower_id(expr.id);
        let kind = match &expr.kind {
            ast::ExprKind::Tuple(inner) => {
                hir::ExprKind::Tuple(inner.iter().map(|child| self.lower_expr(child)).collect())
            }

            ast::ExprKind::Parenthesized(expr) => {
                hir::ExprKind::Parenthesized(Box::new(self.lower_expr(expr)))
            }
            ast::ExprKind::Path(path) => hir::ExprKind::Path(self.resolve_path(path)),
            ast::ExprKind::Call(call) => hir::ExprKind::Call(ayuc_hir::CallExpr {
                callee: Box::new(self.lower_expr(&call.callee)),
                args: call.args.iter().map(|e| self.lower_expr(e)).collect(),
            }),
            ast::ExprKind::Lit(lit) => hir::ExprKind::Lit(match lit {
                ast::Literal::Bool { value } => hir::Literal::Bool(*value),
                ast::Literal::Str { span: _, data } => hir::Literal::Str(*data),
                ast::Literal::InterpolatedStr { span: _, segments } => {
                    hir::Literal::InterpolatedStr(
                        segments
                            .iter()
                            .map(|seg| match seg {
                                ast::IntlSegment::Text(sym) => hir::IntlSegment::Text(*sym),
                                ast::IntlSegment::Var(ident) => {
                                    hir::IntlSegment::Var(self.resolve_ident(ident))
                                }
                            })
                            .collect(),
                    )
                }
                ast::Literal::Integer { span: _, value } => hir::Literal::Integer(*value),
            }),
            ast::ExprKind::Binary(bin) => hir::ExprKind::Binary(hir::BinExpr {
                left: Box::new(self.lower_expr(&bin.left)),
                operator: match bin.operator {
                    ast::Operator::Add => hir::BinaryOp::Add,
                    ast::Operator::Gt => hir::BinaryOp::Gt,
                    ast::Operator::EqualsEquals => hir::BinaryOp::EqualsEquals,
                    ast::Operator::GtOrEqual => hir::BinaryOp::GtOrEqual,
                    ast::Operator::Lt => hir::BinaryOp::Lt,
                    ast::Operator::LtOrEqual => hir::BinaryOp::LtOrEqual,
                    ast::Operator::Minus => hir::BinaryOp::Minus,
                    ast::Operator::NotEquals => hir::BinaryOp::NotEquals,
                    ast::Operator::Mul => hir::BinaryOp::Mul,
                    ast::Operator::Div => hir::BinaryOp::Div,
                    ast::Operator::Modulus => hir::BinaryOp::Modulus,
                },
                right: Box::new(self.lower_expr(&bin.right)),
            }),
        };

        hir::Expr { id, kind }
    }

    fn lower_def(&self, rdef: &RDef) -> hir::Def {
        match rdef {
            RDef::Def(d) => hir::Def::Def(*d),
            RDef::Local(l) => hir::Def::Local(*l),
            RDef::Error => unreachable!(),
        }
    }

    fn resolve_id(&self, id: NodeId) -> hir::Def {
        self.lower_def(&self.rcx.name_resolutions[&id])
    }

    fn resolve_ident(&self, ident: &ast::Ident) -> hir::Def {
        self.resolve_id(ident.id)
    }

    fn resolve_path(&self, path: &ast::Path) -> hir::Path {
        if let Some(qualified) = self.rcx.qualified_paths.get(&path.id) {
            hir::Path {
                target: qualified.last().map(|q| self.lower_def(q)).unwrap(),
                segments: qualified.iter().map(|q| self.lower_def(q)).collect(),
            }
        } else {
            hir::Path {
                target: self.resolve_id(path.id),
                segments: path
                    .segments
                    .iter()
                    .map(|seg| self.resolve_id(seg.id))
                    .collect(),
            }
        }
    }

    fn lower_res(&self, res: &RTy) -> hir::Ty {
        match &res.kind {
            RTyKind::Tuple(inner) => {
                hir::Ty::Tuple(inner.iter().map(|child| self.lower_res(child)).collect())
            }
            RTyKind::Prim(prim) => hir::Ty::Primitive(match prim {
                RPrimTy::Boolean => hir::PrimTy::Boolean,
                RPrimTy::Integer => hir::PrimTy::Integer,
                RPrimTy::Str => hir::PrimTy::Str,
            }),
            RTyKind::Fn(params, return_ty) => hir::Ty::Fn(
                params.iter().map(|id| self.lower_res(id)).collect(),
                Box::new(self.lower_res(return_ty)),
            ),
            RTyKind::Error => unreachable!(),
        }
    }
}
