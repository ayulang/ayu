pub mod predicate;
pub mod raw_token;

use std::str::Chars;

use ayuc_span::Span;
use unicode_properties::UnicodeEmoji;

use crate::raw_token::{InplSegment, LiteralKind, RawToken, RawTokenKind, RawTokenStream};

/// Transforms the input source code to a stream of [RawToken]s.
/// These are basically normal tokens that only contain the token kind and location, no additional data.
pub struct Scanner<'a> {
    chars: Chars<'a>,
    position: usize,
    source_len: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars(),
            position: 0,
            source_len: source.len(),
        }
    }

    pub(crate) fn bump(&mut self) -> Option<char> {
        let c = self.chars.next()?;

        self.position += c.len_utf8();

        Some(c)
    }

    pub(crate) fn eat_while(&mut self, predicate: impl Fn(char) -> bool) {
        while let Some(current) = self.first()
            && predicate(current)
        {
            self.bump();
        }
    }

    pub(crate) fn whitespace(&mut self) -> RawToken {
        let start = self.position - 1;

        self.eat_while(|c| c.is_whitespace());

        RawToken::new(RawTokenKind::Whitespace, (start, self.position))
    }

    pub(crate) fn ident(&mut self, start: usize) -> RawToken {
        self.eat_while(predicate::is_ident_continue);

        match self.first() {
            Some(c) if !c.is_ascii() && c.is_emoji_char() => self.invalid_ident(start),
            _ => RawToken::new(RawTokenKind::Ident, (start, self.position)),
        }
    }

    pub(crate) fn invalid_ident(&mut self, ident_start: usize) -> RawToken {
        self.eat_while(|c| predicate::is_ident_continue(c) || (!c.is_ascii() && c.is_emoji_char()));

        RawToken::new(RawTokenKind::InvalidIdent, (ident_start, self.position))
    }

    pub(crate) fn integer(&mut self) -> RawToken {
        let start = self.position - 1;

        self.eat_while(|c| c.is_ascii_digit());

        let data_span = Span::from(start..self.position);

        RawToken::new(
            RawTokenKind::Literal {
                kind: LiteralKind::Integer { data_span },
            },
            data_span,
        )
    }

    pub(crate) fn string(&mut self) -> RawToken {
        let start = self.position - 1;
        let terminated = self.string_until_termination();

        RawToken::new(
            RawTokenKind::Literal {
                kind: LiteralKind::Str { terminated },
            },
            (start, self.position),
        )
    }

    pub(crate) fn string_until_termination(&mut self) -> bool {
        while let Some(c) = self.bump() {
            match c {
                '"' => return true,
                '\\' if self.first() == Some('\\') || self.first() == Some('"') => {
                    self.bump();
                }
                _ => (),
            }
        }

        false
    }

    pub(crate) fn inpl_text(&mut self, start: usize) -> InplSegment {
        while let Some(c) = self.first() {
            match c {
                '`' | '{' => break,
                '\\' if self.first() == Some('\\') || self.first() == Some('"') => {
                    self.bump();
                    self.bump();
                }
                _ => {
                    self.bump();
                }
            }
        }

        InplSegment::Text {
            span: Span::from((start, self.position)),
        }
    }

    pub(crate) fn inpl_var(&mut self) -> InplSegment {
        let RawToken { kind, span } = self.ident(self.position);
        let invalid = match kind {
            RawTokenKind::Ident => false,
            RawTokenKind::InvalidIdent => true,
            _ => unreachable!(),
        };

        let terminated = match self.first() {
            Some('}') => {
                self.bump();
                true
            }
            _ => false,
        };

        InplSegment::Var {
            span,
            invalid,
            terminated,
        }
    }

    pub(crate) fn interpolated_string(&mut self) -> RawToken {
        let start = self.position - 1;

        let mut terminated = false;
        let mut segments = Vec::new();

        while let Some(c) = self.bump() {
            match c {
                '`' => {
                    terminated = true;

                    break;
                }
                '{' if matches!(self.first(), Some('{')) => {
                    self.bump();

                    segments.push(self.inpl_text(self.position - 2));
                }
                '{' => {
                    segments.push(self.inpl_var());
                }
                '}' if matches!(self.first(), Some('}')) => {
                    self.bump();

                    segments.push(self.inpl_text(self.position - 2));
                }
                '}' => segments.push(InplSegment::InvalidClosing(Span::from((
                    self.position - 1,
                    self.position,
                )))),
                _ => {
                    segments.push(self.inpl_text(self.position - 1));
                }
            }
        }

        RawToken::new(
            RawTokenKind::Literal {
                kind: LiteralKind::InterpolatedStr {
                    terminated,
                    segments,
                },
            },
            (start, self.position),
        )
    }

    pub(crate) fn single(&self, kind: RawTokenKind) -> RawToken {
        RawToken::new(kind, (self.position - 1, self.position))
    }

    pub(crate) fn comment(&mut self) -> RawToken {
        let start = self.position - 1;

        self.bump(); // Slash

        while let Some(c) = self.bump() {
            if c == '\n' {
                break;
            }
        }

        RawToken::new(RawTokenKind::Comment, (start, self.position))
    }

    pub(crate) fn multiline_comment(&mut self) -> RawToken {
        let start = self.position - 1;

        self.bump(); // Asterisk

        while let Some(c) = self.bump() {
            if c == '*'
                && let Some('/') = self.first()
            {
                self.bump();

                break;
            }
        }

        RawToken::new(RawTokenKind::Comment, (start, self.position))
    }

    pub fn next_token(&mut self) -> RawToken {
        let Some(first_char) = self.bump() else {
            return RawToken::new(RawTokenKind::Eof, self.source_len);
        };

        match first_char {
            c if c.is_whitespace() => self.whitespace(),
            c if predicate::is_ident_start(c) => self.ident(self.position - 1),

            '/' if matches!(self.first(), Some('/')) => self.comment(),
            '/' if matches!(self.first(), Some('*')) => self.multiline_comment(),

            ';' => self.single(RawTokenKind::Semi),
            ':' => self.single(RawTokenKind::Colon),
            '+' => self.single(RawTokenKind::Plus),
            '-' => self.single(RawTokenKind::Minus),
            '=' => self.single(RawTokenKind::Equals),
            '(' => self.single(RawTokenKind::OpenParen),
            ')' => self.single(RawTokenKind::CloseParen),
            '{' => self.single(RawTokenKind::OpenBrace),
            '}' => self.single(RawTokenKind::CloseBrace),
            '<' => self.single(RawTokenKind::Lt),
            '>' => self.single(RawTokenKind::Gt),
            ',' => self.single(RawTokenKind::Comma),
            '!' => self.single(RawTokenKind::Exclamation),
            '/' => self.single(RawTokenKind::Slash),
            '%' => self.single(RawTokenKind::Percentage),
            '*' => self.single(RawTokenKind::Asterisk),

            '"' => self.string(),
            '`' => self.interpolated_string(),
            c if c.is_ascii_digit() => self.integer(),

            _ => RawToken::new(RawTokenKind::Unknown, (self.position - 1, self.position)),
        }
    }

    pub fn first(&self) -> Option<char> {
        self.chars.clone().next()
    }

    pub fn second(&self) -> Option<char> {
        let mut chars = self.chars.clone();

        chars.next();

        chars.next()
    }

    pub fn third(&self) -> Option<char> {
        let mut chars = self.chars.clone();

        chars.next();
        chars.next();

        chars.next()
    }
}

impl<'a> From<Scanner<'a>> for RawTokenStream {
    fn from(mut value: Scanner) -> Self {
        let mut tokens = vec![];

        loop {
            let token = value.next_token();
            let is_eof = token.is_eof();

            tokens.push(token);

            if is_eof {
                break;
            }
        }

        Self::new(tokens)
    }
}
