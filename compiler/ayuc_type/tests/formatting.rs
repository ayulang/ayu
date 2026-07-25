use ayuc_type::{
    format::FormatExt,
    interner::TypeInterner,
    ty::{PrimTy, TyKind},
};

#[test]
fn test_formatting() {
    let mut interner = TypeInterner::default();

    let str_id = interner.intern(TyKind::Prim(PrimTy::Str));
    let int_id = interner.intern(TyKind::Prim(PrimTy::Int));
    let bool_id = interner.intern(TyKind::Prim(PrimTy::Bool));

    let tuple_id = interner.intern(TyKind::Tuple(vec![int_id, bool_id]));

    let fun = interner.intern(TyKind::Fn(vec![str_id], tuple_id));
    let fun2 = interner.intern(TyKind::Fn(vec![str_id], str_id));

    assert_eq!(format!("{}", fun.format(&interner)), "(str) -> (int, bool)");
    assert_eq!(format!("{}", fun2.format(&interner)), "(str) -> str");
}
