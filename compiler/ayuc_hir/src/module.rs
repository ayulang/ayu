use std::collections::HashMap;

use ayuc_id::{
    ModuleId,
    ast::NodeId,
    hir::{DefId, HirId},
};
use ayuc_span::symbol::Symbol;
use slotmap::SecondaryMap;

use crate::Item;

/// Contains all items and other relevant definitions of the lowered source file.
#[derive(Debug)]
pub struct Module {
    pub id: ModuleId,
    pub items: SecondaryMap<DefId, Item>,

    pub top_level_items: Vec<DefId>,
    pub items_by_symbol: HashMap<Symbol, DefId>,
    pub id_mappings: HashMap<HirId, NodeId>,
}

impl Module {
    pub fn new(id: ModuleId) -> Self {
        Self {
            id,
            items: SecondaryMap::default(),
            items_by_symbol: HashMap::default(),
            top_level_items: Vec::default(),
            id_mappings: HashMap::default(),
        }
    }

    #[inline]
    pub fn top_items(&self) -> Vec<(DefId, &Item)> {
        self.top_level_items
            .iter()
            .map(|id| (*id, &self.items[*id]))
            .collect()
    }
}
