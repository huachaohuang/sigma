mod error;
pub use error::{Error, Result};

mod token;
use token::*;

mod lexer;
use lexer::Lexer;

pub type Span = std::ops::Range<usize>;

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
