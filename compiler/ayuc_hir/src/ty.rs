#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    Primitive(PrimTy),
    Fn(Vec<Ty>, Box<Ty>),
    Tuple(Vec<Ty>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimTy {
    Boolean,
    Integer,
    Str,
}

impl Ty {
    pub const UNIT: Self = Self::Tuple(Vec::new());
}
