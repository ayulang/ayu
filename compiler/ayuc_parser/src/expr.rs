use ayuc_ast::{BinaryExpression, Call, Expression, Ident, Literal, Operator, expr::Block};
use ayuc_lexer::{
    stream::TokenStream,
    token::{Delimiter, StructuredToken, Token, TokenKind},
};
use ayuc_span::symbol::Symbol;

use crate::{PResult, Parser};

impl Parser<'_> {
    pub fn parse_call_expr(&mut self, prefix: Expression) -> PResult<Call> {
        let tokens = match self.stream.consume() {
            Some(StructuredToken::Delimited(_, Delimiter::Parenthesis, tokens)) => tokens,
            _ => {
                todo!()
            }
        };

        let mut inner = self.branch(TokenStream::new(tokens));
        let mut args = Vec::new();
        let mut expect_expr = true;

        while expect_expr {
            if let Ok(expr) = inner.parse_expression() {
                args.push(expr);
            } else {
                break;
            }

            expect_expr = inner.maybe(TokenKind::Comma);
        }

        Ok(Call {
            callee: Box::new(prefix),
            args,
        })
    }

    pub fn parse_bin_expr(&mut self, left: Expression) -> PResult<BinaryExpression> {
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

    pub fn parse_expr_prefix(&mut self) -> PResult<Expression> {
        let Some(first) = self.stream.consume() else {
            return Err(());
        };

        match first {
            StructuredToken::Token(Token {
                kind: TokenKind::Literal(lit),
                span,
            }) => match lit {
                ayuc_lexer::token::Literal::Str { data_span } => {
                    Ok(Expression::Lit(Literal::Str {
                        span: *span,
                        data: Symbol::intern(&self.source[data_span]),
                    }))
                }
                ayuc_lexer::token::Literal::Integer { data_span } => {
                    let data = &self.source[data_span]; // TODO: PROCESS

                    Ok(Expression::Lit(Literal::Integer {
                        span: *data_span,
                        value: data.parse().unwrap(),
                    }))
                }
            },
            StructuredToken::Token(Token {
                kind: TokenKind::Ident(sym),
                span,
            }) => Ok(Expression::Identifier(Ident {
                span: *span,
                sym: *sym,
            })),
            _ => Err(()),
        }
    }

    pub fn parse_expression(&mut self) -> PResult<Expression> {
        let prefix = self.parse_expr_prefix()?;

        let Some(first) = self.stream.first() else {
            return Ok(prefix);
        };

        match first {
            StructuredToken::Delimited(_, Delimiter::Parenthesis, _) => {
                return Ok(Expression::Call(self.parse_call_expr(prefix)?));
            }
            StructuredToken::Token(Token { kind, .. }) => match kind {
                TokenKind::Plus => return Ok(Expression::Binary(self.parse_bin_expr(prefix)?)),
                _ => {}
            },
            _ => {}
        };

        Ok(prefix)
    }
}
