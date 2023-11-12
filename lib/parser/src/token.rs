use crate::Radix;

pub(crate) enum Token<'a> {
    // Identifier or keyword
    Ident(&'a str),

    // Integer literal
    Int(&'a str, Radix),

    // Floating-point literal
    Float(&'a str),

    // String literal
    Str(&'a str),

    // Punctuation
    Punct(Punct),
}

pub(crate) enum Punct {
    Semi,
    Colon,
    Comma,
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
