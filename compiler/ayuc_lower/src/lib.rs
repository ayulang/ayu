use ayuc_ast as ast;
use ayuc_hir::{self as hir, Package};

use ayuc_id::{
    ast::NodeId,
    hir::{DefIdAllocator, HirId, HirIdAllocator},
};
use ayuc_tyctx::TyCtx;
use bimap::BiHashMap;

pub struct AstLowering<'a> {
    _ty_ctx: &'a mut TyCtx,
    package: hir::Package,
    id_mappings: BiHashMap<NodeId, HirId>,
}

impl<'a> AstLowering<'a> {
    pub fn new(_ty_ctx: &'a mut TyCtx) -> Self {
        let id = _ty_ctx.mint_package_id();

        Self {
            _ty_ctx,
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
        match &item.kind {
            ast::ItemKind::Fn(fn_decl) => hir::Item::Fn(hir::FnItem {
                id: self.package.def_id_allocator.allocate(),
                hir_id: self.lower_id(item.id),
                name: fn_decl.ident.sym,
                return_ty: hir::Ty::None,
                block: self.lower_block(&fn_decl.block),
                params: fn_decl
                    .parameters
                    .parameters
                    .iter()
                    .map(|param| hir::Parameter {
                        name: param.ident.sym,
                    })
                    .collect(),
            }),
            ast::ItemKind::ExternFn(extern_fn) => hir::Item::ExternFn(hir::ExternFnItem {
                id: self.package.def_id_allocator.allocate(),
                hir_id: self.lower_id(item.id),
                name: extern_fn.ident.sym,
            }),
        }
    }

    fn lower_block(&mut self, block: &ast::Block) -> hir::Block {
        hir::Block {
            stmts: block.children.iter().map(|s| self.lower_stmt(s)).collect(),
        }
    }

    fn lower_stmt(&mut self, stmt: &ast::Statement) -> hir::Stmt {
        match stmt {
            ast::Statement::Expr(expr) => hir::Stmt::Expr(self.lower_expr(expr)),
            ast::Statement::Let(var_decl) => hir::Stmt::VarDecl(hir::LetStmt {
                ident: var_decl.ident.sym,
                init: self.lower_expr(&var_decl.init),
            }),
            ast::Statement::Return(ret) => hir::Stmt::Return(hir::ReturnStmt {
                expr: ret.expr.as_ref().map(|expr| self.lower_expr(expr)),
            }),
        }
    }

    fn lower_expr(&mut self, expr: &ast::Expression) -> hir::Expression {
        match expr {
            ast::Expression::Identifier(ident) => hir::Expression::Ident(ident.sym),
            ast::Expression::Call(call) => hir::Expression::Call(ayuc_hir::CallExpr {
                callee: Box::new(self.lower_expr(&call.callee)),
                args: call.args.iter().map(|e| self.lower_expr(e)).collect(),
            }),
            ast::Expression::Lit(lit) => hir::Expression::Lit(match lit {
                ast::Literal::Str { span: _, data } => hir::Literal::Str(*data),
                ast::Literal::Integer { span: _, value } => hir::Literal::Integer(*value),
            }),
            ast::Expression::Binary(bin) => hir::Expression::Binary(hir::BinExpr {
                left: Box::new(self.lower_expr(&bin.left)),
                operator: match bin.operator {
                    ast::Operator::Add => hir::BinaryOp::Add,
                },
                right: Box::new(self.lower_expr(&bin.right)),
            }),
        }
    }
}
