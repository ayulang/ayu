use std::{collections::HashMap, iter};

use ayuc_id::hir::{DefId, LocalId};
use ayuc_span::symbol::Symbol;

use crate::def::Def;

#[derive(Debug, Default)]
pub struct ScopeStack {
    top: Scope,
    stack: Vec<Scope>,
    /// The travelled [DefId]s
    path: Vec<Option<DefId>>,
}

#[derive(Debug, Default)]
pub struct Scope {
    symbols: HashMap<Symbol, Def>,
    absolute_path: Vec<Option<DefId>>,
}

impl ScopeStack {
    pub fn enter(&mut self, def: Option<DefId>) {
        self.path.push(def);
        self.stack.push(Scope::with_path(self.path.clone()));
    }

    pub fn leave(&mut self) {
        self.path.pop();
        self.stack.pop();
    }

    pub fn current_mut(&mut self) -> &mut Scope {
        self.stack.last_mut().unwrap_or(&mut self.top)
    }

    pub fn current(&self) -> &Scope {
        self.stack.last().unwrap_or(&self.top)
    }

    fn lookup_get_scope(&self, sym: Symbol) -> Option<(&Scope, Def)> {
        self.stack
            .iter()
            .rev()
            .chain(std::iter::once(&self.top))
            .find_map(|scope| scope.lookup(sym).map(|d| (scope, d)))
    }

    pub fn lookup(&self, sym: Symbol) -> Option<Def> {
        let (_, def) = self.lookup_get_scope(sym)?;

        Some(def)
    }

    pub fn lookup_path(&self, sym: Symbol) -> Option<(Def, Vec<Def>)> {
        let (scope, def) = self.lookup_get_scope(sym)?;

        Some((def, scope.build_path(def)))
    }

    pub fn register_local(&mut self, sym: Symbol, id: LocalId) {
        self.current_mut().symbols.insert(sym, Def::Local(id));
    }

    /// Registers the [Symbol] in the top scope.
    pub fn register_def(&mut self, sym: Symbol, id: DefId) {
        self.current_mut().symbols.insert(sym, Def::Def(id));
    }
}

impl Scope {
    pub fn with_path(path: Vec<Option<DefId>>) -> Self {
        Self {
            symbols: HashMap::new(),
            absolute_path: path,
        }
    }

    pub fn build_path(&self, target: Def) -> Vec<Def> {
        self.absolute_path
            .iter()
            .flat_map(|def| def.map(Def::Def))
            .chain(iter::once(target))
            .collect()
    }

    #[inline]
    pub fn lookup(&self, sym: Symbol) -> Option<Def> {
        self.symbols.get(&sym).copied()
    }
}
