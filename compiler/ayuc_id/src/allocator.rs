use std::marker::PhantomData;

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
