//! This crate contains identifier definitions for the Ayu compiler.

pub mod allocator;
pub mod ast;
pub mod hir;

#[macro_export]
macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(usize);

        impl $name {
            pub const MAX: Self = Self(usize::MAX);

            pub fn new(value: usize) -> Self {
                Self(value)
            }

            pub fn get(&self) -> usize {
                self.0
            }
        }
    };

    ($name:ident, $allocator:ident) => {
        pub type $allocator = $crate::allocator::IdAllocator<$name>;

        $crate::define_id!($name);

        impl $crate::allocator::Allocatable for $name {
            fn allocate(value: usize) -> Self {
                Self(value)
            }
        }
    };
}
