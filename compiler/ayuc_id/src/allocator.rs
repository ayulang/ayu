use std::{fmt::Debug, marker::PhantomData};

pub trait Allocatable {
    fn allocate(value: usize) -> Self;
}

pub struct IdAllocator<T>
where
    T: Allocatable,
{
    _marker: PhantomData<T>,

    next: usize,
}

impl<T> Debug for IdAllocator<T>
where
    T: Allocatable,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IdAllocator(next={})", self.next)
    }
}

impl<T> IdAllocator<T>
where
    T: Allocatable,
{
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
            next: 0,
        }
    }

    pub fn allocate(&mut self) -> T {
        let next = T::allocate(self.next);

        self.next = self
            .next
            .checked_add(1)
            .expect("reached usize::MAX for id in IdAllocator");

        next
    }
}

impl<T> Default for IdAllocator<T>
where
    T: Allocatable,
{
    fn default() -> Self {
        Self::new()
    }
}
