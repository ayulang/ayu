use ayuc_ast::Path;

use crate::{
    Parser,
    parsable::{Parsable, ParseError, Parsed},
};

impl Parsable for Path {
    const NAME: &str = "path";

    fn parse<'a>(parser: &mut Parser<'a>) -> Result<Parsed<Self>, ParseError> {
        todo!()
    }
}
