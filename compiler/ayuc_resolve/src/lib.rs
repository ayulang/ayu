use std::collections::HashMap;

use ayuc_ast as ast;
use ayuc_hir::{self as hir, PrimTy};
use ayuc_id::ast::NodeId;

pub struct Resolver {
    pub resolutions: HashMap<NodeId, hir::Ty>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            resolutions: HashMap::default(),
        }
    }

    #[inline]
    pub fn resolve(ast: &ast::Ast) -> Self {
        let mut this = Self::new();

        this.resolve_ast(ast);

        this
    }

    pub fn get_res(&self, id: NodeId) -> hir::Ty {
        self.resolutions.get(&id).copied().unwrap_or(hir::Ty::Error)
    }

    pub fn resolve_ast(&mut self, ast: &ast::Ast) {
        for item in &ast.items {
            self.resolve_item(item);
        }
    }

    pub fn resolve_item(&mut self, item: &ast::Item) {
        match &item.kind {
            ast::ItemKind::Fn(fun) => {
                for param in &fun.parameters.parameters {
                    self.resolve_ty(&param.ty);
                }

                self.resolve_ty(&fun.return_ty);
            }
            ast::ItemKind::ExternFn(extern_fun) => {
                for param in &extern_fun.parameters.parameters {
                    self.resolve_ty(&param.ty);
                }

                self.resolve_ty(&extern_fun.return_ty);
            }
        }
    }

    pub fn resolve_ty(&mut self, ty: &ast::Ty) {
        let resolved = match &ty.kind {
            ast::TyKind::Unit => hir::Ty::Unit,
            ast::TyKind::Path(p) => {
                if p.segments.len() == 1 {
                    let segment = &p.segments[0];

                    if let Some(prim) = PrimTy::from_name(segment.ident.sym) {
                        hir::Ty::Primitive(prim)
                    } else {
                        hir::Ty::Error
                    }
                } else {
                    hir::Ty::Error // not yet.
                }
            }
        };

        self.resolutions.insert(ty.id, resolved);
    }
}
