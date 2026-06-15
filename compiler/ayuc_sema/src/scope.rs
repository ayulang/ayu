use std::collections::HashMap;

use ayuc_hir::{DefId, HirId, PackageId};
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
    pub scopes: Vec<Scope>,

    pub top_level_scopes: HashMap<PackageId, ScopeId>,
    pub hir_scopes: HashMap<HirId, ScopeId>,
}

impl Scope {
    pub fn parent<'a>(&'a self, ctx: &'a ScopeCtx) -> Option<&'a Scope> {
        self.parent.map(|p| ctx.scope(p))
    }
}

impl ScopeCtx {
    fn create_scope_and_register(&mut self, parent: Option<ScopeId>) -> ScopeId {
        let id = ScopeId(self.scopes.len());
        let scope = Scope {
            id,
            definitions: HashMap::new(),
            parent,
        };

        self.scopes.push(scope);

        id
    }

    pub fn register(&mut self, scope_id: ScopeId, symbol: Symbol, def_id: DefId) -> Option<DefId> {
        self.scope_mut(scope_id).definitions.insert(symbol, def_id)
    }

    pub fn enter_scope(&mut self, parent: ScopeId) -> ScopeId {
        self.create_scope_and_register(Some(parent))
    }

    pub fn enter_top_level_scope(&mut self, package_id: PackageId) -> ScopeId {
        let id = self.create_scope_and_register(None);

        self.top_level_scopes.insert(package_id, id);

        id
    }

    pub fn top_level_scope(&self, package_id: PackageId) -> Option<&Scope> {
        self.top_level_scopes
            .get(&package_id)
            .map(|s| self.scope(*s))
    }

    pub fn hir_scope(&self, hir_id: HirId) -> Option<&Scope> {
        let scope_id = self.hir_scope_id(hir_id)?;

        Some(self.scope(scope_id))
    }

    pub fn hir_scope_id(&self, hir_id: HirId) -> Option<ScopeId> {
        self.hir_scopes.get(&hir_id).cloned()
    }

    pub fn attach_scope(&mut self, scope_id: ScopeId, hir_id: HirId) {
        self.hir_scopes.insert(hir_id, scope_id);
    }

    pub fn scope(&self, scope_id: ScopeId) -> &Scope {
        &self.scopes[scope_id.0]
    }

    pub fn scope_mut(&mut self, scope_id: ScopeId) -> &mut Scope {
        &mut self.scopes[scope_id.0]
    }

    pub fn lookup(&self, mut scope: ScopeId, sym: Symbol) -> Option<DefId> {
        loop {
            let current = self.scope(scope);

            if let Some(&def) = current.definitions.get(&sym) {
                return Some(def);
            }

            scope = current.parent?;
        }
    }
}
