use ayuc_hir::{Item, Package};

use crate::scope::ScopeCtx;

pub struct BuildScopePass<'a> {
    pub scope_ctx: &'a mut ScopeCtx,
}

impl<'a> BuildScopePass<'a> {
    pub fn new(scope_ctx: &'a mut ScopeCtx) -> Self {
        Self { scope_ctx }
    }

    pub fn run(&mut self, package: &Package) {
        if self.scope_ctx.top_level_scope(package.id).is_some() {
            eprintln!("scope is already built");

            return;
        }

        let top_level = self.scope_ctx.enter_top_level_scope(package.id);

        // todo: diagnostics instead of shadowing
        for item in &package.items {
            match item {
                Item::ExternFn(extern_fn) => {
                    self.scope_ctx
                        .register(top_level, extern_fn.name, extern_fn.id);
                }
                Item::Fn(function) => {
                    self.scope_ctx
                        .register(top_level, function.name, function.id);

                    let scope_id = self.scope_ctx.enter_scope(top_level);

                    self.scope_ctx.attach_scope(scope_id, function.hir_id);
                }
            }
        }
    }
}
