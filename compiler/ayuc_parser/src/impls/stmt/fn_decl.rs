use ayuc_ir::node::{leaf::ident::Ident, stmt::fn_decl::FnDecl};

use crate::{
    Parser,
    parsable::{Parsable, ParseResult},
};

impl Parsable for FnDecl {
    fn parse<'a>(parser: &mut Parser<'a>) -> ParseResult<'a, Self> {
        let ident = parser.expect::<Ident>()?;

        println!("{:#?}", ident);

        todo!()
    }
}
