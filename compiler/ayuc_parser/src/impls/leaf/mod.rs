use ayuc_ir::node::leaf::ident::Ident;

use crate::{
    Parser,
    parsable::{Parsable, ParseResult},
};

impl Parsable for Ident {
    fn parse<'a>(input: &mut Parser<'a>) -> ParseResult<'a, Self> {
        let next = input.stream.consume();
    }
}
