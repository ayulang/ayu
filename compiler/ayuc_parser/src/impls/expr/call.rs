use ayuc_ast::node::{
    expr::call::{ArgumentList, CallExpression},
    leaf::ident::Ident,
};

use crate::{
    Parser,
    parsable::{Assertable, Parsable, Parsed},
};

impl Parsable for CallExpression {
    fn parse<'a>(parser: &mut Parser<'a>) -> Result<Parsed<Self>, ()> {
        let ident = parser.assert_parsable::<Ident>()?;
        let arguments = parser.assert_parsable::<ArgumentList>()?;

        Ok(Parsed::Present(Self {
            callee: ident,
            arguments,
        }))
    }
}

impl Parsable for ArgumentList {
    fn parse<'a>(parser: &mut Parser<'a>) -> Result<Parsed<Self>, ()> {
        parser.stream.consume();

        Ok(Parsed::Present(Self()))
    }
}

impl Assertable for CallExpression {
    const NAME: &str = "call expression";
}

impl Assertable for ArgumentList {
    const NAME: &str = "argument list";
}
