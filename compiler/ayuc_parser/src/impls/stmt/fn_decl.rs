use ayuc_ir::node::stmt::fn_decl::FnDecl;

use crate::{
    Parser,
    parsable::{Parsable, ParseResult},
};

impl Parsable for FnDecl {
    fn parse<'a>(input: &mut Parser<'a>) -> ParseResult<'a, Self> {
        todo!()
    }
}
