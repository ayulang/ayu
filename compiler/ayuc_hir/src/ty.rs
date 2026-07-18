#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    Unit,
    Primitive(PrimTy),
    Fn(Vec<Ty>, Box<Ty>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimTy {
    Boolean,
    Integer,
    Str,
}
