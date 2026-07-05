#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ty {
    Unit,
    Primitive(PrimTy),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimTy {
    Integer,
    Str,
}
