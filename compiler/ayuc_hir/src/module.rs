use std::collections::HashMap;

use ayuc_id::{
    BodyId, ModuleId,
    ast::NodeId,
    hir::{DefId, HirId},
};
use ayuc_span::symbol::Symbol;
use slotmap::SlotMap;

use crate::Stmt;

/// Contains all items and other relevant definitions of the lowered source file.
#[derive(Debug)]
pub struct Module {
    pub id: ModuleId,
    pub top_level_items: Vec<DefId>,
    pub items_by_symbol: HashMap<Symbol, DefId>,
    pub id_mappings: HashMap<HirId, NodeId>,

    pub bodies: SlotMap<BodyId, Vec<Stmt>>,
}

impl Module {
    pub fn new(id: ModuleId) -> Self {
        Self {
            id,
            items_by_symbol: HashMap::default(),
            top_level_items: Vec::default(),
            id_mappings: HashMap::default(),
            bodies: SlotMap::default(),
        }
    }
}
