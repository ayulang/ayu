use std::collections::{HashMap, HashSet};

use ayuc_id::{
    ast::NodeId,
    hir::{DefId, LocalId},
};
use ayuc_item::Item;
use ayuc_type::interner::TypeInterner;
use slotmap::SlotMap;

use crate::local::LocalInfo;

#[derive(Default)]
pub struct Session {
    pub interner: TypeInterner,

    pub items: SlotMap<DefId, Item>,

    pub locals: SlotMap<LocalId, LocalInfo>,
    pub locals_by_id: HashMap<NodeId, LocalId>,

    pub synthetics: HashSet<NodeId>,
}

impl Session {
    pub fn register_local(&mut self, info: LocalInfo) -> LocalId {
        let node_id = info.id;
        let id = self.locals.insert(info);

        self.locals_by_id.insert(node_id, id);

        id
    }
}
