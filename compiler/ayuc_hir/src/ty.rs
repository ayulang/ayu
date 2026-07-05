use ayuc_span::symbol::Symbol;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ty {
    Unit,
    Primitive(PrimTy),
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimTy {
    Integer,
    Str,
}

impl PrimTy {
    pub fn from_name(sym: Symbol) -> Option<Self> {
        let prim = match sym.as_str() {
            "int" => Self::Integer,
            "str" => Self::Str,
            _ => return None,
        };

        Some(prim)
    }
}
