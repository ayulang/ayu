use ayuc_ast::{LetStmt, ReturnStmt, Stmt, StmtKind};
use ayuc_lexer::token::{Keyword, StructuredToken, Token, TokenKind};

use crate::{PResult, Parser};

impl Parser<'_, '_> {
    pub fn parse_let_stmt(&mut self) -> PResult<LetStmt> {
        if !self.maybe(TokenKind::Keyword(Keyword::Let)) {
            return Err(crate::DummyError);
        }

        let ident = self.parse_ident().unwrap();

        if !self.maybe(TokenKind::Equals) {
            return Err(crate::DummyError);
        }

        let expr = self.parse_expression()?;

        Ok(LetStmt { ident, init: expr })
    }

    pub fn parse_return_stmt(&mut self) -> PResult<ReturnStmt> {
        if !self.maybe(TokenKind::Keyword(Keyword::Return)) {
            return Err(crate::DummyError);
        }

        let expr = self.parse_expression()?; // make it try to parse an expression instead.

        Ok(ReturnStmt { expr: Some(expr) })
    }

    pub fn parse_statement(&mut self) -> PResult<Stmt> {
        let Some(first) = self.stream.first() else {
            return Err(crate::DummyError);
        };

        let snapshot = self.stream.snapshot();

        let kind = match first {
            StructuredToken::Token(Token {
                kind: TokenKind::Keyword(Keyword::Return),
                ..
            }) => StmtKind::Return(self.parse_return_stmt()?),

            StructuredToken::Token(Token {
                kind: TokenKind::Keyword(Keyword::Let),
                ..
            }) => StmtKind::Let(self.parse_let_stmt()?),

            _ => self
                .with_rollback(|p| p.parse_expression())
                .map(StmtKind::Expr)?,
        };

        Ok(Stmt {
            id: self.node_id_allocator.allocate(),
            span: self.stream.span_since(snapshot),
            kind,
        })
    }
}
