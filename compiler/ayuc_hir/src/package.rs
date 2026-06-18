use ayuc_id::hir::{DefIdAllocator, HirIdAllocator, PackageId};

use crate::item::Item;

#[derive(Debug)]
pub struct Package {
    pub id: PackageId,
    pub items: Vec<Item>,

    pub hir_id_allocator: HirIdAllocator,
    pub def_id_allocator: DefIdAllocator,
}
