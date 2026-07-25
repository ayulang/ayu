use ayuc_type::{
    interner::TypeInterner,
    ty::{PrimTy, TyKind},
};

#[test]
fn test_primitives() {
    let mut interner = TypeInterner::default();

    let int_id1 = interner.intern(TyKind::Prim(PrimTy::Int));
    let int_id2 = interner.intern(TyKind::Prim(PrimTy::Int));
    let bool_id = interner.intern(TyKind::Prim(PrimTy::Bool));

    assert_eq!(int_id1, int_id2);
    assert_ne!(int_id1, bool_id);

    assert_eq!(interner.get(int_id1), &TyKind::Prim(PrimTy::Int));
    assert_eq!(interner.get(bool_id), &TyKind::Prim(PrimTy::Bool));
}

#[test]
fn test_tuple() {
    let mut interner = TypeInterner::default();

    let int_id = interner.intern(TyKind::Prim(PrimTy::Int));
    let bool_id = interner.intern(TyKind::Prim(PrimTy::Bool));

    let tuple_id1 = interner.intern(TyKind::Tuple(vec![int_id, bool_id]));
    let tuple_id2 = interner.intern(TyKind::Tuple(vec![int_id, bool_id]));
    let different_tuple = interner.intern(TyKind::Tuple(vec![bool_id, int_id]));

    assert_eq!(tuple_id1, tuple_id2);
    assert_ne!(tuple_id1, different_tuple);

    assert_eq!(
        interner.get(tuple_id1),
        &TyKind::Tuple(vec![int_id, bool_id])
    );
}

#[test]
fn test_fn() {
    let mut interner = TypeInterner::default();

    let int_id = interner.intern(TyKind::Prim(PrimTy::Int));
    let str_id = interner.intern(TyKind::Prim(PrimTy::Str));

    let fn_id1 = interner.intern(TyKind::Fn(vec![int_id], str_id));
    let fn_id2 = interner.intern(TyKind::Fn(vec![int_id], str_id));
    let fn_id3 = interner.intern(TyKind::Fn(vec![str_id, int_id], int_id));
    let fn_id4 = interner.intern(TyKind::Fn(vec![str_id, int_id], int_id));

    assert_eq!(fn_id1, fn_id2);
    assert_ne!(fn_id1, fn_id3);
    assert_eq!(fn_id3, fn_id4);

    assert_eq!(interner.get(fn_id1), &TyKind::Fn(vec![int_id], str_id));
}
