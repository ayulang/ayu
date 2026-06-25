use ayuc_ast::{BinaryExpression, Call, Expression, Literal, Operator, expr::Block};
use ayuc_lexer::{
    stream::TokenStream,
    token::{Delimiter, StructuredToken, Token, TokenKind},
};
use ayuc_span::symbol::Symbol;

use crate::{PResult, Parser};

impl Parser<'_> {
    pub fn parse_call_expr(&mut self) -> PResult<Call> {
        let ident = self.parse_ident()?;

        let tokens = match self.stream.consume() {
            Some(StructuredToken::Delimited(_, Delimiter::Parenthesis, tokens)) => tokens,
            _ => {
                todo!()
            }
        };

        let mut inner = self.branch(TokenStream::new(tokens));
        let mut args = Vec::new();

        while !inner.stream.is_exhausted() {
            args.push(inner.parse_expression()?);
        }

        Ok(Call {
            callee: Box::new(Expression::Identifier(ident)),
            args,
        })
    }

    pub fn parse_bin_expr(&mut self) -> PResult<BinaryExpression> {
        let left = self.parse_expression()?;

        let operator = if self.maybe(TokenKind::Plus) {
            Operator::Add
        } else {
            return Err(());
        };

        let right = self.parse_expression()?;

        Ok(BinaryExpression {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }

    pub fn parse_block_expr(&mut self) -> PResult<Block> {
        let (span, tokens) = match self.stream.consume() {
            Some(StructuredToken::Delimited(span, Delimiter::Braces, tokens)) => (*span, tokens),
            _ => {
                todo!()
            }
        };

        let mut inner = self.branch(TokenStream::new(tokens));
        let mut children = Vec::new();

        while !inner.stream.is_exhausted() {
            children.push(inner.parse_statement()?);
        }

        Ok(Block { span, children })
    }

    pub fn parse_expression(&mut self) -> PResult<Expression> {
        let Some(first) = self.stream.first() else {
            return Err(());
        };

        match first {
            StructuredToken::Token(Token {
                kind: TokenKind::Literal(ayuc_lexer::token::Literal::Str { data_span }),
                span,
            }) => {
                let expr = Expression::Lit(Literal::Str {
                    span: *span,
                    data: Symbol::intern(&self.source[data_span]),
                });

                self.stream.consume();

                Ok(expr)
            }
            StructuredToken::Token(Token {
                kind: TokenKind::Ident(_),
                ..
            }) if matches!(
                self.stream.second(),
                Some(StructuredToken::Delimited(_, Delimiter::Parenthesis, _))
            ) =>
            {
                Ok(Expression::Call(self.parse_call_expr()?))
            }
            StructuredToken::Token(Token {
                kind: TokenKind::Ident(_),
                ..
            }) if matches!(
                self.stream.second(),
                Some(StructuredToken::Token(Token {
                    kind: TokenKind::Plus,
                    ..
                }))
            ) =>
            {
                Ok(Expression::Binary(self.parse_bin_expr()?))
            }
            StructuredToken::Token(Token {
                kind: TokenKind::Ident(_),
                ..
            }) => Ok(Expression::Identifier(self.parse_ident().unwrap())),
            _ => todo!("{first:#?}"),
        }
    }
}
