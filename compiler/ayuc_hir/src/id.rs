macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(usize);

        impl $name {
            pub fn new(id: usize) -> Self {
                Self(id)
            }

            pub fn get(&self) -> usize {
                self.0
            }
        }
    };
}

macro_rules! define_allocator {
    ($id:ident, $allocator_name:ident) => {
        #[derive(Debug)]
        pub struct $allocator_name(usize);

        impl $allocator_name {
            pub fn new() -> Self {
                Self(0)
            }

            pub fn allocate(&mut self) -> $id {
                self.0 += 1;

                $id(self.0 - 1)
            }
        }

        impl Default for $allocator_name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

define_id!(HirId);
define_id!(PackageId);
define_id!(DefId);

define_allocator!(HirId, HirIdAllocator);
