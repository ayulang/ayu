use ayuc_ast as ast;
use ayuc_hir::{self as hir, Package};

use ayuc_id::{
    ast::NodeId,
    hir::{DefIdAllocator, HirId, HirIdAllocator},
};
use ayuc_resolve::Resolver;
use ayuc_tyctx::TyCtx;
use bimap::BiHashMap;

pub struct AstLowering<'a> {
    _ty_ctx: &'a mut TyCtx,
    resolver: &'a Resolver,
    package: hir::Package,
    id_mappings: BiHashMap<NodeId, HirId>,
}

impl<'a> AstLowering<'a> {
    pub fn new(_ty_ctx: &'a mut TyCtx, resolver: &'a Resolver) -> Self {
        let id = _ty_ctx.mint_package_id();

        Self {
            _ty_ctx,
            resolver,
            package: Package {
                id,
                items: Vec::new(),
                def_id_allocator: DefIdAllocator::default(),
                hir_id_allocator: HirIdAllocator::default(),
            },
            id_mappings: BiHashMap::new(),
        }
    }

    #[must_use]
    pub fn lower(mut self, ast: &ayuc_ast::Ast) -> Package {
        for item in &ast.items {
            let item = self.lower_item(item);

            self.package.items.push(item);
        }

        self.package
    }

    #[must_use]
    fn lower_id(&mut self, id: NodeId) -> HirId {
        if self.id_mappings.get_by_left(&id).is_some() {
            panic!("tried to lower NodeId ({id:?}) into HirId: it has already been lowered");
        }

        let hir_id = self.package.hir_id_allocator.allocate();

        self.id_mappings.insert(id, hir_id);

        hir_id
    }

    fn lower_item(&mut self, item: &ast::Item) -> hir::Item {
        let id = self.package.def_id_allocator.allocate();
        let hir_id = self.lower_id(item.id);

        let kind = match &item.kind {
            ast::ItemKind::Fn(fun) => hir::ItemKind::Fn(hir::FnItem {
                name: fun.ident.sym,
                block: self.lower_block(&fun.block),
                return_ty: self.lower_ty(&fun.return_ty),
                params: fun
                    .parameters
                    .parameters
                    .iter()
                    .map(|p| hir::Parameter {
                        name: p.ident.sym,
                        ty: self.lower_ty(&p.ty),
                    })
                    .collect(),
            }),
            ast::ItemKind::ExternFn(extern_fun) => hir::ItemKind::ExternFn(hir::ExternFnItem {
                name: extern_fun.ident.sym,
                return_ty: self.lower_ty(&extern_fun.return_ty),
                params: extern_fun
                    .parameters
                    .parameters
                    .iter()
                    .map(|p| hir::Parameter {
                        name: p.ident.sym,
                        ty: self.lower_ty(&p.ty),
                    })
                    .collect(),
            }),
        };

        hir::Item { id, hir_id, kind }
    }

    fn lower_block(&mut self, block: &ast::Block) -> hir::Block {
        hir::Block {
            stmts: block.children.iter().map(|s| self.lower_stmt(s)).collect(),
        }
    }

    fn lower_stmt(&mut self, stmt: &ast::Stmt) -> hir::Stmt {
        let id = self.lower_id(stmt.id);
        let kind = match &stmt.kind {
            ast::StmtKind::Expr(expr) => hir::StmtKind::Expr(self.lower_expr(expr)),
            ast::StmtKind::Let(decl) => hir::StmtKind::Let(hir::LetStmt {
                ident: decl.ident.sym,
                init: self.lower_expr(&decl.init),
            }),
            ast::StmtKind::Return(ret) => hir::StmtKind::Return(hir::ReturnStmt {
                expr: ret.expr.as_ref().map(|expr| self.lower_expr(expr)),
            }),
        };

        hir::Stmt { id, kind }
    }

    fn lower_expr(&mut self, expr: &ast::Expr) -> hir::Expr {
        let id = self.lower_id(expr.id);
        let kind = match &expr.kind {
            ast::ExprKind::Identifier(ident) => hir::ExprKind::Ident(ident.sym),
            ast::ExprKind::Call(call) => hir::ExprKind::Call(ayuc_hir::CallExpr {
                callee: Box::new(self.lower_expr(&call.callee)),
                args: call.args.iter().map(|e| self.lower_expr(e)).collect(),
            }),
            ast::ExprKind::Lit(lit) => hir::ExprKind::Lit(match lit {
                ast::Literal::Str { span: _, data } => hir::Literal::Str(*data),
                ast::Literal::Integer { span: _, value } => hir::Literal::Integer(*value),
            }),
            ast::ExprKind::Binary(bin) => hir::ExprKind::Binary(hir::BinExpr {
                left: Box::new(self.lower_expr(&bin.left)),
                operator: match bin.operator {
                    ast::Operator::Add => hir::BinaryOp::Add,
                },
                right: Box::new(self.lower_expr(&bin.right)),
            }),
        };

        hir::Expr { id, kind }
    }

    // TODO
    fn lower_ty(&mut self, ty: &ast::Ty) -> hir::Ty {
        self.resolver.get_res(ty.id)
    }
}
