use std::str::Chars;

pub mod token;

pub struct Lexer<'a> {
    source: Chars<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source.chars(),
        }
    }
}
