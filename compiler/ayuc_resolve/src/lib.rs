pub mod name_res;
pub mod scope;
pub mod type_res;

use std::collections::HashMap;

use ayuc_ast as ast;
use ayuc_hir as hir;
use ayuc_id::{
    ast::NodeId,
    hir::{DefId, LocalId},
};
use slotmap::SlotMap;

use crate::scope::ScopeStack;

pub struct Resolver {
    /// Stores the resolved HIR `Ty`s of AST `Ty`s
    pub ty_resolutions: HashMap<NodeId, hir::Ty>,

    /// Stores the resolved `Def`s (local or item definitions) of identifiers.
    pub name_resolutions: HashMap<NodeId, hir::Def>,

    /// The assigned [DefId] for identifiers of items.
    pub def_ids: SlotMap<DefId, NodeId>,
    pub defs_by_node: HashMap<NodeId, DefId>,

    /// The assigned [LocalId] for `let` statements.
    pub locals: SlotMap<LocalId, NodeId>,
    pub locals_by_node: HashMap<NodeId, LocalId>,

    /// For the name resolver.
    stack: ScopeStack,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            ty_resolutions: HashMap::default(),
            name_resolutions: HashMap::default(),
            def_ids: SlotMap::default(),
            defs_by_node: HashMap::default(),
            locals: SlotMap::default(),
            locals_by_node: HashMap::default(),
            stack: ScopeStack::new(),
        }
    }

    /// Constructs a new [Resolver], performs name and type resolution and returns the [Resolver].
    #[inline]
    pub fn resolve(ast: &ast::Ast) -> Self {
        let mut this = Self::new();

        this.resolve_names(ast);
        this.resolve_types(ast);

        this
    }

    pub fn get_ty_res(&self, id: NodeId) -> hir::Ty {
        self.ty_resolutions
            .get(&id)
            .copied()
            .unwrap_or(hir::Ty::Error)
    }

    pub fn get_name_res(&self, id: NodeId) -> hir::Def {
        self.name_resolutions
            .get(&id)
            .copied()
            .unwrap_or(hir::Def::Error)
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}
