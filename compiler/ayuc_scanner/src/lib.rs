pub mod predicate;
pub mod raw_token;

use std::str::Chars;

use unicode_properties::UnicodeEmoji;

use crate::raw_token::{LiteralKind, RawToken, RawTokenKind};

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
        let c = self.chars.next();

        if !c.is_none() {
            self.position += 1;
        }

        c
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

    pub(crate) fn ident(&mut self) -> RawToken {
        let start = self.position - 1;

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

    pub(crate) fn single(&self, kind: RawTokenKind) -> RawToken {
        RawToken::new(kind, (self.position - 1, self.position))
    }

    pub fn next_token(&mut self) -> RawToken {
        let Some(first_char) = self.bump() else {
            return RawToken::new(RawTokenKind::Eof, self.source_len);
        };

        match first_char {
            c if c.is_whitespace() => self.whitespace(),
            c if predicate::is_ident_start(c) => self.ident(),

            ';' => self.single(RawTokenKind::Semi),
            '(' => self.single(RawTokenKind::OpenParen),
            ')' => self.single(RawTokenKind::CloseParen),
            '{' => self.single(RawTokenKind::OpenBrace),
            '}' => self.single(RawTokenKind::CloseBrace),

            '"' => self.string(),

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
