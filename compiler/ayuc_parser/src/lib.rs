pub mod expr;
pub mod item;
pub mod path;
pub mod stmt;
pub mod ty;

use ayuc_ast::{Ast, Ident, Parameter, ParameterList};
use ayuc_diagnostic::DiagnosticContext;
use ayuc_id::ast::NodeIdAllocator;
use ayuc_lexer::{
    stream::TokenStream,
    token::{Delimiter, StructuredToken, Token, TokenKind},
};

#[derive(Debug)]
pub struct DummyError;

pub type PResult<T> = Result<T, DummyError>;

/// Used for parsing an input file into an abstract syntax tree.
pub struct Parser<'src, 'ctx> {
    /// The input token stream.
    pub(crate) stream: TokenStream<'src>,

    file_id: usize,
    source: &'src str,
    node_id_allocator: NodeIdAllocator,
    dcx: &'ctx mut DiagnosticContext,
}

impl<'src, 'ctx> Parser<'src, 'ctx> {
    pub fn new(
        dcx: &'ctx mut DiagnosticContext,
        file_id: usize,
        source: &'src str,
        stream: TokenStream<'src>,
    ) -> Self {
        Self {
            stream,
            file_id,
            source,
            node_id_allocator: NodeIdAllocator::default(),
            dcx,
        }
    }

    pub fn branch<'b>(&'b mut self, stream: TokenStream<'src>) -> Parser<'src, 'b> {
        Parser {
            stream,
            file_id: self.file_id,
            source: self.source,
            node_id_allocator: self.node_id_allocator.clone(),
            dcx: self.dcx,
        }
    }

    pub(crate) fn maybe(&mut self, k: TokenKind) -> bool {
        if let Some(StructuredToken::Token(Token { kind, .. })) = self.stream.first()
            && *kind == k
        {
            self.stream.consume();

            true
        } else {
            false
        }
    }

    pub fn parse_ident(&mut self) -> PResult<Ident> {
        if let Some(StructuredToken::Token(Token {
            kind: TokenKind::Ident(sym),
            span,
        })) = self.stream.consume()
        {
            Ok(Ident {
                sym: *sym,
                span: *span,
            })
        } else {
            Err(DummyError)
        }
    }

    pub fn parse_parameter(&mut self) -> PResult<Parameter> {
        let ident = self.parse_ident().unwrap();

        if !self.maybe(TokenKind::Colon) {
            return Err(DummyError);
        }

        let ty = self.parse_ty().unwrap();

        Ok(Parameter { ident, ty })
    }

    pub fn parse_parameter_list(&mut self) -> PResult<ParameterList> {
        let snapshot = self.stream.snapshot();

        let tokens = match self.stream.consume() {
            Some(StructuredToken::Delimited(_, Delimiter::Parenthesis, tokens)) => tokens,
            _ => {
                todo!()
            }
        };

        let mut parameters = Vec::new();

        if tokens.is_empty() {
            return Ok(ParameterList {
                span: self.stream.span_since(snapshot),
                parameters,
            });
        }

        let mut inner = self.branch(TokenStream::new(tokens));
        let mut expect_param = true;

        while expect_param {
            parameters.push(inner.parse_parameter()?);

            expect_param = inner.maybe(TokenKind::Comma);
        }

        Ok(ParameterList {
            span: self.stream.span_since(snapshot),
            parameters,
        })
    }

    pub fn with_rollback<F, T>(&mut self, parse_fn: F) -> PResult<T>
    where
        F: FnOnce(&mut Self) -> PResult<T>,
    {
        let snapshot = self.stream.snapshot();
        let res = parse_fn(self);

        if res.is_err() {
            self.stream.restore(snapshot);
        }

        res
    }

    pub fn parse_full(mut self) -> Option<Ast> {
        let mut items = Vec::new();

        while !self.stream.is_exhausted()
            && !matches!(
                self.stream.first(),
                Some(StructuredToken::Token(Token {
                    kind: TokenKind::Eof,
                    ..
                }))
            )
        {
            match self.parse_item() {
                Ok(node) => items.push(node),
                Err(_) => return None,
            }
        }

        Some(Ast { items })
    }
}
