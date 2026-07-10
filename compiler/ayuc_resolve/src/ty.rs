use ayuc_span::symbol::Symbol;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ty {
    Unit,
    Prim(PrimTy),
    Error,
}

impl Ty {
    pub fn get_name(&self) -> &'static str {
        match self {
            Self::Prim(ty) => match ty {
                PrimTy::Integer => "int",
                PrimTy::Str => "str",
            },
            Self::Unit => "unit",
            Self::Error => unreachable!(),
        }
    }
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
