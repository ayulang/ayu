use ayuc_ast::{BinExpr, CallExpr, Expr, ExprKind, Ident, Literal, Operator, expr::Block};
use ayuc_lexer::{
    stream::TokenStream,
    token::{Delimiter, StructuredToken, Token, TokenKind},
};
use ayuc_span::symbol::Symbol;

use crate::{PResult, Parser};

impl Parser<'_, '_> {
    pub fn parse_call_expr(&mut self, prefix: Expr) -> PResult<CallExpr> {
        let tokens = match self.stream.consume() {
            Some(StructuredToken::Delimited(_, Delimiter::Parenthesis, tokens)) => tokens,
            _ => return Err(crate::DummyError),
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

        Ok(CallExpr {
            callee: Box::new(prefix),
            args,
        })
    }

    pub fn parse_bin_expr(&mut self, left: Expr) -> PResult<BinExpr> {
        let operator = if self.maybe(TokenKind::Plus) {
            Operator::Add
        } else {
            return Err(crate::DummyError);
        };

        let right = self.parse_expression()?;

        Ok(BinExpr {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }

    pub fn parse_block_expr(&mut self) -> PResult<Block> {
        let (span, tokens) = match self.stream.consume() {
            Some(StructuredToken::Delimited(span, Delimiter::Braces, tokens)) => (*span, tokens),
            _ => return Err(crate::DummyError),
        };

        let mut inner = self.branch(TokenStream::new(tokens));
        let mut children = Vec::new();

        while !inner.stream.is_exhausted() {
            children.push(inner.parse_statement()?);
        }

        Ok(Block { span, children })
    }

    pub fn parse_expr_prefix(&mut self) -> PResult<Expr> {
        let Some(first) = self.stream.consume() else {
            return Err(crate::DummyError);
        };

        Ok(match first {
            StructuredToken::Token(Token {
                kind: TokenKind::Literal(lit),
                span,
            }) => {
                let kind = match lit {
                    ayuc_lexer::token::Literal::Str { data_span } => Literal::Str {
                        span: *span,
                        data: Symbol::intern(&self.source[data_span]),
                    },
                    ayuc_lexer::token::Literal::Integer { data_span } => {
                        let data = &self.source[data_span];

                        Literal::Integer {
                            span: *span,
                            value: data.parse().map_err(|_| crate::DummyError)?,
                        }
                    }
                };

                Expr {
                    span: *span,
                    id: self.node_id_allocator.allocate(),
                    kind: ExprKind::Lit(kind),
                }
            }
            StructuredToken::Token(Token {
                kind: TokenKind::Ident(sym),
                span,
            }) => Expr {
                span: *span,
                id: self.node_id_allocator.allocate(),
                kind: ExprKind::Identifier(Ident {
                    span: *span,
                    sym: *sym,
                }),
            },
            _ => return Err(crate::DummyError),
        })
    }

    pub fn parse_expression(&mut self) -> PResult<Expr> {
        let prefix = self.parse_expr_prefix()?;

        let Some(first) = self.stream.first() else {
            return Ok(prefix);
        };

        match first {
            StructuredToken::Delimited(span, Delimiter::Parenthesis, _) => {
                return Ok(Expr {
                    span: prefix.span.merged(*span),
                    id: self.node_id_allocator.allocate(),
                    kind: ExprKind::Call(self.parse_call_expr(prefix)?),
                });
            }
            StructuredToken::Token(Token { kind, .. }) if *kind == TokenKind::Plus => {
                let bin = self.parse_bin_expr(prefix)?;

                return Ok(Expr {
                    span: bin.left.span.merged(bin.right.span),
                    id: self.node_id_allocator.allocate(),
                    kind: ExprKind::Binary(bin),
                });
            }

            _ => {}
        };

        Ok(prefix)
    }
}
