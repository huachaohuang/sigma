mod error;
pub use error::{Error, Result};

mod token;
use token::*;

mod lexer;
use lexer::Lexer;

pub type Span = std::ops::Range<usize>;

#[derive(Clone, Debug)]
pub enum Radix {
    Bin,
    Oct,
    Dec,
    Hex,
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            lexer: Lexer::new(input),
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<(Span, Token<'a>)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.lexer.next()
    }
}
