macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

define_id!(PackageId);
define_id!(TypeRefId);
define_id!(DefId);
