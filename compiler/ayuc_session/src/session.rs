use std::collections::HashSet;

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
    pub synthetics: HashSet<NodeId>,
}
