use ayuc_ast::{
    Call, Expression,
    expr::{Block, Ident},
};
use ayuc_lexer::{
    stream::TokenStream,
    token::{Delimiter, StructuredToken, Token, TokenKind},
};
use ayuc_span::Span;

use crate::{
    Parser,
    parsable::{Parsable, ParseError, Parsed},
};

impl Parsable for Call {
    const NAME: &str = "call expression";

    fn parse<'a>(parser: &mut Parser<'a>) -> Result<Parsed<Self>, ParseError> {
        let ident = parser.assert_parsable::<Ident>()?;

        let snapshot = parser.stream.snapshot();

        let tokens = match parser.stream.consume() {
            Some(StructuredToken::Delimited(_, Delimiter::Parenthesis, tokens)) => tokens,
            _ => {
                return Ok(Parsed::Missing(
                    parser.stream.span_since(snapshot).end.into(),
                ));
            }
        };

        let mut inner = parser.branch(TokenStream::new(tokens));
        let mut args = Vec::new();

        while !inner.stream.is_exhausted() {
            args.push(inner.parse_expression()?);
        }

        Ok(Parsed::Present(Self {
            callee: Box::new(Expression::Identifier(ident)),
            args,
        }))
    }
}

impl Parsable for Block {
    const NAME: &str = "block expression";

    fn parse<'a>(parser: &mut Parser<'a>) -> Result<Parsed<Self>, ParseError> {
        let snapshot = parser.stream.snapshot();

        let (span, tokens) = match parser.stream.consume() {
            Some(StructuredToken::Delimited(span, Delimiter::Braces, tokens)) => (*span, tokens),
            _ => {
                return Ok(Parsed::Missing(Span::from(
                    parser.stream.past_span_or_distance(2, snapshot).end,
                )));
            }
        };

        let mut inner = parser.branch(TokenStream::new(tokens));
        let mut children = Vec::new();

        while !inner.stream.is_exhausted() {
            children.push(inner.parse_statement()?);
        }

        Ok(Parsed::Present(Self { children, span }))
    }
}

impl Parsable for Ident {
    const NAME: &str = "identifier";

    fn parse<'a>(parser: &mut Parser<'a>) -> Result<Parsed<Self>, ParseError> {
        let snapshot = parser.stream.snapshot();

        if let Some(StructuredToken::Token(Token {
            kind: TokenKind::Ident(sym),
            span,
        })) = parser.stream.consume()
        {
            Ok(Parsed::Present(Self {
                sym: *sym,
                span: *span,
            }))
        } else {
            Ok(Parsed::Missing(
                parser.stream.past_span_or_distance(1, snapshot),
            ))
        }
    }
}
