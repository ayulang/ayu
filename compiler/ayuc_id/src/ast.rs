use crate::allocator::{Allocatable, IdAllocator};

pub type NodeIdAllocator = IdAllocator<NodeId>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

impl NodeId {
    pub const MAX: Self = Self(usize::MAX);

    pub fn new(value: usize) -> Self {
        Self(value)
    }

    pub fn get(&self) -> usize {
        self.0
    }
}

impl Allocatable for NodeId {
    fn allocate(value: usize) -> Self {
        Self::new(value)
    }
}
