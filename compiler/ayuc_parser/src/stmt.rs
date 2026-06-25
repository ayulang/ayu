use ayuc_ast::{LetStatement, ReturnStatement, Statement};
use ayuc_lexer::token::{Delimiter, Keyword, StructuredToken, Token, TokenKind};

use crate::{PResult, Parser};

impl Parser<'_> {
    pub fn parse_let_stmt(&mut self) -> PResult<LetStatement> {
        if !self.maybe(TokenKind::Keyword(Keyword::Let)) {
            return Err(());
        }

        let ident = self.parse_ident().unwrap();

        if !self.maybe(TokenKind::Equals) {
            return Err(());
        }

        let expr = self.parse_expression()?;

        Ok(LetStatement { ident, init: expr })
    }

    pub fn parse_return_stmt(&mut self) -> PResult<ReturnStatement> {
        if !self.maybe(TokenKind::Keyword(Keyword::Return)) {
            return Err(());
        }

        let expr = self.parse_expression()?; // make it try to parse an expression instead.

        Ok(ReturnStatement { expr: Some(expr) })
    }

    pub fn parse_statement(&mut self) -> PResult<Statement> {
        let Some(first) = self.stream.first() else {
            return Err(());
        };

        // redo this so it tries known patterns first, and then maybe expressions?
        match first {
            StructuredToken::Token(Token {
                kind: TokenKind::Ident(_),
                ..
            }) if matches!(
                self.stream.second(),
                Some(StructuredToken::Delimited(_, Delimiter::Parenthesis, _))
            ) =>
            {
                let prefix = self.parse_expr_prefix()?;

                Ok(Statement::Expr(ayuc_ast::Expression::Call(
                    self.parse_call_expr(prefix)?,
                )))
            }
            StructuredToken::Token(Token {
                kind: TokenKind::Keyword(Keyword::Return),
                ..
            }) => Ok(Statement::Return(self.parse_return_stmt()?)),
            StructuredToken::Token(Token {
                kind: TokenKind::Keyword(Keyword::Let),
                ..
            }) => Ok(Statement::Let(self.parse_let_stmt()?)),
            _ => todo!("{first:?}"),
        }
    }
}
