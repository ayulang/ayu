use std::collections::HashMap;

use ayuc_id::hir::{DefId, LocalId};
use ayuc_span::symbol::Symbol;

use crate::def::Def;

#[derive(Debug)]
pub struct ScopeStack {
    top: Scope,
    stack: Vec<Scope>,
}

#[derive(Debug)]
pub struct Scope {
    symbols: HashMap<Symbol, Def>,
}

impl ScopeStack {
    pub fn new() -> Self {
        Self {
            top: Scope::new(),
            stack: Vec::new(),
        }
    }

    pub fn enter(&mut self) {
        self.stack.push(Scope::new());
    }

    pub fn leave(&mut self) {
        self.stack.pop();
    }

    pub fn current_mut(&mut self) -> &mut Scope {
        self.stack.last_mut().unwrap_or(&mut self.top)
    }

    pub fn lookup(&self, sym: Symbol) -> Option<Def> {
        self.stack
            .iter()
            .rev()
            .chain(std::iter::once(&self.top))
            .find_map(|scope| scope.symbols.get(&sym).copied())
    }

    pub fn register_local(&mut self, sym: Symbol, id: LocalId) {
        self.current_mut().symbols.insert(sym, Def::Local(id));
    }

    /// Registers the [Symbol] in the top scope.
    pub fn register_def(&mut self, sym: Symbol, id: DefId) {
        self.top.symbols.insert(sym, Def::Def(id));
    }
}

impl Scope {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }
}

impl Default for ScopeStack {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}
