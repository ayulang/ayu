use std::{cell::Cell, fmt::Debug, marker::PhantomData, rc::Rc};

pub trait Allocatable {
    fn allocate(value: usize) -> Self;
}

#[derive(Clone)]
pub struct IdAllocator<T>
where
    T: Allocatable,
{
    _marker: PhantomData<T>,

    next: Rc<Cell<usize>>,
}

impl<T> Debug for IdAllocator<T>
where
    T: Allocatable,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IdAllocator(next={})", self.next.get())
    }
}

impl<T> IdAllocator<T>
where
    T: Allocatable,
{
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
            next: Rc::new(Cell::new(0)),
        }
    }

    pub fn allocate(&self) -> T {
        let id = self.next.get();

        self.next.set(
            id.checked_add(1)
                .expect("reached usize::MAX for id in IdAllocator"),
        );

        T::allocate(id)
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
