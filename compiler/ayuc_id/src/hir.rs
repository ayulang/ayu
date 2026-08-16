use slotmap::new_key_type;

new_key_type! { pub struct HirId; }
new_key_type! { pub struct DefId; }
new_key_type! { pub struct LocalId; }
new_key_type! { pub struct PackageId; }
