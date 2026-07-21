use std::collections::HashSet;

use ayuc_id::{
    ast::NodeId,
    hir::{DefId, LocalId},
};
use slotmap::SlotMap;

use crate::{ItemInfo, local::LocalInfo};

#[derive(Default)]
pub struct Session {
    items: SlotMap<DefId, ItemInfo>,
    locals: SlotMap<LocalId, LocalInfo>,

    synthetics: HashSet<NodeId>,
}

impl Session {
    pub fn mark_as_synthetic(&mut self, id: NodeId) {
        self.synthetics.insert(id);
    }

    pub fn is_synthetic(&self, id: NodeId) -> bool {
        self.synthetics.contains(&id)
    }

    pub fn register_item(&mut self, info: ItemInfo) -> DefId {
        self.items.insert(info)
    }

    pub fn register_local(&mut self, info: LocalInfo) -> LocalId {
        self.locals.insert(info)
    }

    pub fn item(&self, id: DefId) -> &ItemInfo {
        &self.items[id]
    }

    pub fn local(&self, id: LocalId) -> &LocalInfo {
        &self.locals[id]
    }
}
