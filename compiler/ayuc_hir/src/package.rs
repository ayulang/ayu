use ayuc_id::hir::{DefId, HirIdAllocator, LocalId, PackageId};
use slotmap::SecondaryMap;

use crate::{Local, item::Item};

#[derive(Debug)]
pub struct Package {
    pub id: PackageId,

    pub items: SecondaryMap<DefId, Item>,
    pub locals: SecondaryMap<LocalId, Local>,

    pub hir_id_allocator: HirIdAllocator,
}

impl Package {
    pub fn new(id: PackageId) -> Self {
        Self {
            id,
            items: SecondaryMap::new(),
            locals: SecondaryMap::new(),
            hir_id_allocator: HirIdAllocator::new(),
        }
    }
}
