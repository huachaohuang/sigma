use std::fmt;

use crate::Radix;

#[derive(Clone, Debug)]
pub(crate) enum Token<'a> {
    // End of input
    End,

    // Identifier or keyword
    Ident(&'a str),

    // String literal
    Str(&'a str),

    // Integer literal
    Int(&'a str, Radix),

    // Floating-point literal
    Float(&'a str),

    // Punctuation
    Punct(Punct),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Punct {
    Semi,
    Colon,
    Comma,
    Dot,
    DotDot,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Eq,
    EqEq,
    Not,
    NotEq,
    LAngle,
    LAngleEq,
    RAngle,
    RAngleEq,
    Plus,
    PlusEq,
    Minus,
    MinusEq,
    Star,
    StarEq,
    Slash,
    SlashEq,
    Percent,
    PercentEq,
}

impl Punct {
    pub(crate) const fn as_str(&self) -> &'static str {
        use Punct::*;
        match self {
            Semi => ";",
            Colon => ":",
            Comma => ",",
            Dot => ".",
            DotDot => "..",
            LParen => "(",
            RParen => ")",
            LBrace => "{",
            RBrace => "}",
            LBracket => "[",
            RBracket => "]",
            Eq => "=",
            EqEq => "==",
            Not => "!",
            NotEq => "!=",
            LAngle => "<",
            LAngleEq => "<=",
            RAngle => ">",
            RAngleEq => ">=",
            Plus => "+",
            PlusEq => "+=",
            Minus => "-",
            MinusEq => "-=",
            Star => "*",
            StarEq => "*=",
            Slash => "/",
            SlashEq => "/=",
            Percent => "%",
            PercentEq => "%=",
        }
    }
}

impl fmt::Display for Punct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
