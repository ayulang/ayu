use std::collections::HashSet;

use ayuc_ast::{self as ast, Ast, ExternModItem, ModItem};
use ayuc_hir::{self as hir, Module};
use ayuc_id::{ModuleId, ast::NodeId, hir::HirId};
use ayuc_item::{self as item};
use ayuc_resolve::{Def as RDef, ResolutionContext};
use ayuc_session::Session;
use slotmap::SlotMap;

pub struct AstLowerer<'sc> {
    module: Module,

    rcx: &'sc ResolutionContext,
    sess: &'sc mut Session,

    hir_ids: SlotMap<HirId, NodeId>,
    already_lowered: HashSet<NodeId>,
}

impl<'sc> AstLowerer<'sc> {
    pub fn new(id: ModuleId, rcx: &'sc ResolutionContext, sess: &'sc mut Session) -> Self {
        Self {
            module: Module::new(id),

            rcx,
            sess,

            hir_ids: SlotMap::with_key(),
            already_lowered: HashSet::default(),
        }
    }

    pub fn lower(mut self, ast: &Ast) -> Module {
        self.lower_ast(ast);

        self.module
    }

    fn lower_ast(&mut self, ast: &Ast) {
        for item in &ast.items {
            self.lower_item(item);

            self.module
                .top_level_items
                .push(self.rcx.defs_by_node[&item.id]);
        }
    }

    fn lower_item(&mut self, item: &ast::Item) {
        let def_id = self.rcx.defs_by_node[&item.id];
        let hir_id = self.lower_id(item.id);

        self.sess.items[def_id].hir_id = Some(hir_id);
        self.module
            .items_by_symbol
            .insert(self.sess.items[def_id].name(), def_id);

        match &item.kind {
            ast::ItemKind::Fn(fn_item) => {
                let body = self.lower_body(&fn_item.block.children);

                let item::ItemKind::Fn(sess_fn) = &mut self.sess.items[def_id].kind else {
                    unreachable!()
                };

                let body_id = self.module.bodies.insert(body);

                sess_fn.body_id = Some(body_id);
            }
            ast::ItemKind::InlineMod(ModItem { items, .. })
            | ast::ItemKind::ExternMod(ExternModItem { items, .. }) => {
                for item in items {
                    self.lower_item(item);
                }
            }
            ast::ItemKind::ExternFn(_) => {}
            ast::ItemKind::FileMod(_) => {}
        }
    }

    #[must_use]
    fn lower_id(&mut self, id: NodeId) -> HirId {
        if self.already_lowered.contains(&id) {
            panic!("one NodeId cannot have multiple HirIds");
        }

        let hir_id = self.hir_ids.insert(id);

        self.module.id_mappings.insert(hir_id, id);
        self.already_lowered.insert(id);

        hir_id
    }

    #[must_use]
    fn lower_body(&mut self, statements: &[ast::Stmt]) -> Vec<hir::Stmt> {
        statements.iter().map(|s| self.lower_stmt(s)).collect()
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

    fn lower_block(&mut self, block: &ast::Block) -> hir::Block {
        hir::Block {
            stmts: block.children.iter().map(|s| self.lower_stmt(s)).collect(),
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
                ty: self.rcx.ty_id_of(stmt.id),
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
                ast::Literal::Float { span: _, value } => hir::Literal::Float(*value),
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
}
