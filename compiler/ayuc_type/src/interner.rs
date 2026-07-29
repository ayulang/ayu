use std::collections::HashMap;

use ayuc_id::TyId;
use slotmap::SlotMap;

use crate::ty::TyKind;

/// An interner used to intern [TyKind]s and get a [TyId] back for reference.
#[derive(Default)]
pub struct TypeInterner {
    pool: SlotMap<TyId, TyKind>,
    map: HashMap<TyKind, TyId>,
}

impl TypeInterner {
    pub fn intern(&mut self, ty: TyKind) -> TyId {
        if let Some(&id) = self.map.get(&ty) {
            return id;
        }

        let id = self.pool.insert(ty.clone());

        self.map.insert(ty, id);

        id
    }

    pub fn get(&self, id: TyId) -> &TyKind {
        &self.pool[id]
    }
}
