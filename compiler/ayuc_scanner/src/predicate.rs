pub fn is_ident_start(c: char) -> bool {
    c == '_' || unicode_ident::is_xid_start(c)
}

pub fn is_ident_continue(c: char) -> bool {
    unicode_ident::is_xid_continue(c)
}
