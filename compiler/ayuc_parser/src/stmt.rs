use ayuc_ast::{
    AssignOperator, AssignStmt, IfStmt, LetStmt, LoopStmt, ReturnStmt, Stmt, StmtKind, WhileStmt,
};
use ayuc_diagnostic::{Diagnostic, Label, colored::Colorize};
use ayuc_lexer::token::{Keyword, StructuredToken, Token, TokenKind};

use crate::{PResult, Parser};

impl Parser<'_, '_> {
    pub fn parse_if_stmt(&mut self) -> PResult<IfStmt> {
        if !self.maybe(TokenKind::Keyword(Keyword::If)) {
            unreachable!()
        }

        let expr = self.parse_expression()?;
        let block = self.parse_block_expr()?;

        Ok(IfStmt { expr, block })
    }

    pub fn parse_while_stmt(&mut self) -> PResult<WhileStmt> {
        if !self.maybe(TokenKind::Keyword(Keyword::While)) {
            unreachable!()
        }

        let expr = self.parse_expression()?;
        let block = self.parse_block_expr()?;

        Ok(WhileStmt { expr, block })
    }

    pub fn parse_loop_stmt(&mut self) -> PResult<LoopStmt> {
        if !self.maybe(TokenKind::Keyword(Keyword::Loop)) {
            unreachable!()
        }

        let block = self.parse_block_expr()?;

        Ok(LoopStmt { block })
    }

    pub fn parse_let_stmt(&mut self) -> PResult<LetStmt> {
        let snapshot = self.stream.snapshot();

        if !self.maybe(TokenKind::Keyword(Keyword::Let)) {
            todo!()
        }

        let mutable = self.maybe(TokenKind::Keyword(Keyword::Mut));
        let ident = self.parse_ident()?;

        if !self.maybe(TokenKind::Colon) {
            let span = self.stream.span_since(snapshot);

            return Err(Diagnostic::error(self.file_id, span)
                .with_message("`let` statements require a type annotation")
                .with_label(Label::primary(span, "doesn't have a type annotation"))
                .with_help(format!(
                    "add a type annotation in the form of {}",
                    format!("`let {}: SomeType = ...`", ident.sym.as_str())
                        .as_str()
                        .bright_green()
                )));
        }

        let ty = self.parse_ty()?;

        if !self.maybe(TokenKind::Equals) {
            todo!()
        }
        let expr = self.parse_expression()?;

        Ok(LetStmt {
            ty,
            mutable,
            ident,
            init: expr,
        })
    }

    pub fn parse_return_stmt(&mut self) -> PResult<ReturnStmt> {
        if !self.maybe(TokenKind::Keyword(Keyword::Return)) {
            todo!()
        }

        let expr = self.with_rollback(|this| this.parse_expression()).ok();

        Ok(ReturnStmt { expr })
    }

    pub fn parse_assign_stmt(&mut self) -> PResult<AssignStmt> {
        let ident = self.parse_ident()?;
        let operator = match self.require_token()? {
            StructuredToken::Token(Token {
                kind: TokenKind::Equals,
                ..
            }) => AssignOperator::Assign,
            StructuredToken::Token(Token {
                kind: TokenKind::PlusEquals,
                ..
            }) => AssignOperator::Add,
            StructuredToken::Token(Token {
                kind: TokenKind::MinusEquals,
                ..
            }) => AssignOperator::Subtract,
            _ => todo!(),
        };

        let value = self.parse_expression()?;

        Ok(AssignStmt {
            ident,
            operator,
            value,
        })
    }

    pub fn parse_statement(&mut self) -> PResult<Stmt> {
        let Some(first) = self.stream.first() else {
            todo!()
        };

        let snapshot = self.stream.snapshot();

        let kind = match first {
            StructuredToken::Token(Token {
                kind: TokenKind::Keyword(Keyword::If),
                ..
            }) => StmtKind::If(self.parse_if_stmt()?),

            StructuredToken::Token(Token {
                kind: TokenKind::Keyword(Keyword::Break),
                ..
            }) => {
                self.stream.consume();

                StmtKind::Break
            }

            StructuredToken::Token(Token {
                kind: TokenKind::Keyword(Keyword::While),
                ..
            }) => StmtKind::While(self.parse_while_stmt()?),

            StructuredToken::Token(Token {
                kind: TokenKind::Keyword(Keyword::Loop),
                ..
            }) => StmtKind::Loop(self.parse_loop_stmt()?),

            StructuredToken::Token(Token {
                kind: TokenKind::Keyword(Keyword::Return),
                ..
            }) => StmtKind::Return(self.parse_return_stmt()?),

            StructuredToken::Token(Token {
                kind: TokenKind::Keyword(Keyword::Let),
                ..
            }) => StmtKind::Let(self.parse_let_stmt()?),

            StructuredToken::Token(Token {
                kind: TokenKind::Ident(_),
                ..
            }) if matches!(
                self.stream.second(),
                Some(StructuredToken::Token(Token {
                    kind: TokenKind::Equals | TokenKind::PlusEquals | TokenKind::MinusEquals,
                    ..
                }))
            ) =>
            {
                StmtKind::Assignment(self.parse_assign_stmt()?)
            }

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
