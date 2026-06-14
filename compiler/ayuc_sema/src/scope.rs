use std::collections::HashMap;

use ayuc_hir::DefId;
use ayuc_span::symbol::Symbol;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(usize);

#[derive(Debug)]
pub struct Scope {
    pub id: ScopeId,
    pub definitions: HashMap<Symbol, DefId>,
    pub parent: Option<ScopeId>,
}

#[derive(Debug, Default)]
pub struct ScopeCtx {
    pub scopes: HashMap<ScopeId, Scope>,

    next_scope_id: usize,
}

impl ScopeId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

impl Scope {
    pub fn parent<'a>(&'a self, ctx: &'a ScopeCtx) -> Option<&'a Scope> {
        self.parent.and_then(|p| ctx.scopes.get(&p))
    }

    pub fn lookup(&self, ctx: &ScopeCtx, sym: Symbol) -> Option<DefId> {
        let mut current = self;

        loop {
            if let Some(&def_id) = current.definitions.get(&sym) {
                return Some(def_id);
            }

            current = current.parent(ctx)?;
        }
    }
}

impl ScopeCtx {
    pub fn mint_id(&mut self) -> ScopeId {
        self.next_scope_id += 1;

        ScopeId(self.next_scope_id - 1)
    }
}
