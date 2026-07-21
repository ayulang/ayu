use ayuc_ast::{
    AlternateBranch, AssignOperator, AssignStmt, Expr, ExprKind, IfStmt, LetStmt, LoopStmt,
    ReturnStmt, Stmt, StmtKind, WhileStmt,
};
use ayuc_diagnostic::{Diagnostic, Label, Recovery};
use ayuc_lexer::token::{Keyword, StructuredToken, Token, TokenKind};

use crate::{PResult, Parser};

impl Parser<'_, '_, '_> {
    pub fn parse_if_stmt(&mut self) -> PResult<IfStmt> {
        if !self.maybe(TokenKind::Keyword(Keyword::If)) {
            unreachable!()
        }

        let expr = self.parse_expression()?;
        let block = self.parse_block()?;
        let alternate = if self.maybe(TokenKind::Keyword(Keyword::Else)) {
            if let Some(StructuredToken::Token(Token {
                kind: TokenKind::Keyword(Keyword::If),
                ..
            })) = self.stream.first()
            {
                Some(AlternateBranch::Another(Box::new(self.parse_if_stmt()?)))
            } else {
                Some(AlternateBranch::Final(self.parse_block()?))
            }
        } else {
            None
        };

        Ok(IfStmt {
            expr,
            block,
            alternate,
        })
    }

    pub fn parse_while_stmt(&mut self) -> PResult<WhileStmt> {
        if !self.maybe(TokenKind::Keyword(Keyword::While)) {
            unreachable!()
        }

        let expr = self.parse_expression()?;
        let block = self.parse_block()?;

        Ok(WhileStmt { expr, block })
    }

    pub fn parse_loop_stmt(&mut self) -> PResult<LoopStmt> {
        if !self.maybe(TokenKind::Keyword(Keyword::Loop)) {
            unreachable!()
        }

        let block = self.parse_block()?;

        Ok(LoopStmt { block })
    }

    pub fn parse_let_stmt(&mut self) -> PResult<LetStmt> {
        let snapshot = self.stream.snapshot();

        if !self.maybe(TokenKind::Keyword(Keyword::Let)) {
            todo!()
        }

        let mutable = self.maybe(TokenKind::Keyword(Keyword::Mut));
        let ident = self.parse_ident()?;

        let ty = if self.maybe(TokenKind::Colon) {
            Some(self.parse_ty()?)
        } else {
            None
        };

        if !self.maybe(TokenKind::Equals) {
            let span = self.stream.span_since(snapshot);

            return Err(Diagnostic::error(self.file_id, span, Recovery::Fatal)
                .with_message("variables must be initialized with a value")
                .with_label(Label::primary(span, "uninitialized variable")));
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
        let snapshot = self.stream.snapshot();

        if !self.maybe(TokenKind::Keyword(Keyword::Return)) {
            todo!()
        }

        let expr = self
            .with_rollback(|this| this.parse_expression())
            .ok()
            .unwrap_or_else(|| {
                let id = self.node_id_allocator.allocate();

                self.sess.mark_as_synthetic(id);

                Expr {
                    id,
                    kind: ExprKind::UNIT,
                    span: self.stream.span_since(snapshot),
                }
            });

        Ok(ReturnStmt { expr })
    }

    pub fn parse_assign_stmt(&mut self) -> PResult<AssignStmt> {
        let ident = self.parse_ident()?;
        let kind = match self.require_token()? {
            StructuredToken::Token(Token { kind, .. }) => kind,
            StructuredToken::Delimited(..) => unreachable!(),
        };

        let operator = match kind {
            TokenKind::Equals => AssignOperator::Assign,
            TokenKind::PlusEquals => AssignOperator::Add,
            TokenKind::MinusEquals => AssignOperator::Subtract,
            TokenKind::SlashEquals => AssignOperator::Div,
            TokenKind::AsteriskEquals => AssignOperator::Mul,
            TokenKind::PercentageEquals => AssignOperator::Modulus,
            _ => unreachable!(),
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
                    kind: TokenKind::Equals
                        | TokenKind::PlusEquals
                        | TokenKind::MinusEquals
                        | TokenKind::SlashEquals
                        | TokenKind::PercentageEquals
                        | TokenKind::AsteriskEquals,
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
