use ayuc_ast::{
    BinExpr, CallExpr, Expr, ExprKind, Ident, IntlSegment, Literal, Operator, expr::Block,
};
use ayuc_diagnostic::{Diagnostic, Label};
use ayuc_lexer::{
    stream::TokenStream,
    token::{Delimiter, StructuredToken, Token, TokenKind},
};
use ayuc_span::{Span, symbol::Symbol};

use crate::{PResult, Parser};

/// The precedence of a call expression.
const CALL_PRECEDENCE: usize = 15;

/// A list of tokens mapped to operators with their precedence.
const OPERATORS: [(TokenKind, Operator, usize); 11] = [
    (TokenKind::Asterisk, Operator::Mul, 13),
    (TokenKind::Slash, Operator::Div, 13),
    (TokenKind::Percentage, Operator::Modulus, 13),
    (TokenKind::Plus, Operator::Add, 12),
    (TokenKind::Minus, Operator::Minus, 12),
    (TokenKind::Lt, Operator::Lt, 10),
    (TokenKind::LtOrEqual, Operator::LtOrEqual, 10),
    (TokenKind::Gt, Operator::Gt, 10),
    (TokenKind::GtOrEqual, Operator::GtOrEqual, 10),
    (TokenKind::EqualsEquals, Operator::EqualsEquals, 9),
    (TokenKind::NotEquals, Operator::NotEquals, 9),
];

impl Parser<'_, '_> {
    pub fn parse_expr_prefix(&mut self) -> PResult<Expr> {
        let snapshot = self.stream.snapshot();
        let first = self.require_token()?;

        Ok(match first {
            StructuredToken::Delimited(span, Delimiter::Parenthesis, tokens) => {
                let mut inner = self.branch(TokenStream::new(tokens));
                let expr = inner.parse_expression()?;

                if !inner.stream.is_exhausted() {
                    let first = match inner.stream.first().unwrap() {
                        StructuredToken::Token(Token { span, .. })
                        | StructuredToken::Delimited(span, ..) => *span,
                    };

                    let span = Span::from((first.start, span.end));

                    return Err(Diagnostic::error(self.file_id, span)
                        .with_message("leftover tokens in parenthesized expression")
                        .with_label(Label::help(expr.span, "the parsed inner expression"))
                        .with_label(Label::primary(span, "the leftover tokens")));
                }

                Expr {
                    id: self.node_id_allocator.allocate(),
                    kind: ExprKind::Parenthesized(Box::new(expr)),
                    span: *span,
                }
            }
            StructuredToken::Token(Token {
                kind: TokenKind::Literal(lit),
                span,
            }) => {
                let kind = match lit {
                    ayuc_lexer::token::Literal::Bool { value } => Literal::Bool { value: *value },
                    ayuc_lexer::token::Literal::Str { data_span } => Literal::Str {
                        span: *span,
                        data: Symbol::intern(&self.source[data_span]),
                    },
                    ayuc_lexer::token::Literal::InterpolatedString { span, segments } => {
                        Literal::InterpolatedStr {
                            span: *span,
                            segments: segments
                                .iter()
                                .map(|seg| match seg {
                                    ayuc_lexer::token::InplSegment::Text { span } => {
                                        IntlSegment::Text(Symbol::intern(&self.source[span]))
                                    }
                                    ayuc_lexer::token::InplSegment::Var { span } => {
                                        IntlSegment::Var(Ident {
                                            id: self.node_id_allocator.allocate(),
                                            span: *span,
                                            sym: Symbol::intern(&self.source[span]),
                                        })
                                    }
                                })
                                .collect(),
                        }
                    }
                    ayuc_lexer::token::Literal::Integer { data_span } => {
                        let data = &self.source[data_span];

                        Literal::Integer {
                            span: *span,
                            value: data.parse().unwrap(),
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
                kind: TokenKind::Ident(_),
                span,
            }) => {
                self.stream.restore(snapshot);

                Expr {
                    span: *span,
                    id: self.node_id_allocator.allocate(),
                    kind: ExprKind::Path(self.parse_path()?),
                }
            }
            _ => todo!(),
        })
    }

    pub fn parse_block(&mut self) -> PResult<Block> {
        let (span, tokens) = match self.stream.consume() {
            Some(StructuredToken::Delimited(span, Delimiter::Braces, tokens)) => (*span, tokens),
            _ => todo!(),
        };

        let mut inner = self.branch(TokenStream::new(tokens));
        let mut children = Vec::new();

        while !inner.stream.is_exhausted() {
            children.push(inner.parse_statement()?);
        }

        Ok(Block { span, children })
    }

    fn parse_call_expr(&mut self, prefix: Expr) -> PResult<CallExpr> {
        let tokens = match self.stream.consume() {
            Some(StructuredToken::Delimited(_, Delimiter::Parenthesis, tokens)) => tokens,
            _ => todo!(),
        };

        let mut args = Vec::new();

        if !tokens.is_empty() {
            let mut inner = self.branch(TokenStream::new(tokens));
            let mut expect_expr = true;

            while expect_expr {
                if let Ok(expr) = inner.parse_expression() {
                    args.push(expr);
                } else {
                    break;
                }

                expect_expr = inner.maybe(TokenKind::Comma);
            }
        }

        Ok(CallExpr {
            callee: Box::new(prefix),
            args,
        })
    }

    fn parse_expression_with_prec(&mut self, min_prec: usize) -> PResult<Expr> {
        let mut left = self.parse_expr_prefix()?;

        while let Some(token) = self.stream.first() {
            // Check if it is a call expression and check it against `min_prec`
            if matches!(
                token,
                StructuredToken::Delimited(_, Delimiter::Parenthesis, _)
            ) {
                if CALL_PRECEDENCE < min_prec {
                    break;
                }

                let left_span = left.span;
                let snapshot = self.stream.snapshot();
                let parsed = self.parse_call_expr(left)?;

                left = Expr {
                    span: left_span.merged(self.stream.span_since(snapshot)),
                    id: self.node_id_allocator.allocate(),
                    kind: ExprKind::Call(parsed),
                };

                continue;
            }

            // Handle binary expressions
            let info = match token {
                StructuredToken::Token(Token { kind, .. }) => OPERATORS
                    .iter()
                    .find(|(k, _, _)| k == kind)
                    .map(|(_, operator, prec)| (*operator, *prec)),
                _ => None,
            };

            let Some((operator, prec)) = info else {
                break;
            };

            if prec < min_prec {
                break;
            }

            self.stream.consume();

            let right = self.parse_expression_with_prec(prec + 1)?;

            left = Expr {
                span: left.span.merged(right.span),
                id: self.node_id_allocator.allocate(),
                kind: ExprKind::Binary(BinExpr {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                }),
            };
        }

        Ok(left)
    }

    pub fn parse_expression(&mut self) -> PResult<Expr> {
        self.parse_expression_with_prec(0)
    }
}
