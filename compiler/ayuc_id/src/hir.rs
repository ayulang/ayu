use crate::define_id;

define_id!(HirId, HirIdAllocator);
define_id!(PackageId);

define_id!(DefId, DefIdAllocator); // will be changed later to allow multiple files
