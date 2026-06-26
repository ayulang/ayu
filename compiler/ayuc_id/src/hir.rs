use slotmap::new_key_type;

use crate::define_id;

define_id!(HirId, HirIdAllocator);
define_id!(PackageId);

new_key_type! { pub struct DefId; }
new_key_type! { pub struct LocalId; }
