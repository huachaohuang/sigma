use std::str::CharIndices;

use crate::token::*;
use crate::{Error, Radix, Result, Span};

pub(crate) struct Lexer<'a> {
    input: &'a str,
    chars: CharIndices<'a>,
    backup: Option<(usize, char)>,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.char_indices(),
            backup: None,
        }
    }

    fn get(&mut self) -> Option<(usize, char)> {
        self.backup.take().or_else(|| self.chars.next())
    }

    fn eat(&mut self, mut f: impl FnMut(char) -> bool) -> Option<(usize, char)> {
        self.get().and_then(|(i, c)| {
            if f(c) {
                Some((i, c))
            } else {
                self.put(i, c);
                None
            }
        })
    }

    fn put(&mut self, i: usize, c: char) {
        self.backup = Some((i, c));
    }

    fn slice(&self, span: Span) -> &'a str {
        // SAFETY: `span` is always valid for internal use.
        unsafe { self.input.get_unchecked(span) }
    }

    fn skip_while(&mut self, mut f: impl FnMut(char) -> bool) -> Option<(usize, char)> {
        while let Some((i, c)) = self.get() {
            if !f(c) {
                return Some((i, c));
            }
        }
        None
    }

    fn parse_str(&mut self, start: usize) -> Result<(Span, Token<'a>)> {
        while let Some((i, c)) = self.get() {
            match c {
                '"' => return Ok((start..i + 1, Token::Str(self.slice(start + 1..i)))),
                '\\' => {
                    let _ = self.get();
                }
                _ => {}
            }
        }
        Err(Error::invalid_token(
            start..self.input.len(),
            "unterminated string",
        ))
    }

    fn parse_num(&mut self, start: usize, first: char) -> Result<(Span, Token<'a>)> {
        if first == '0' {
            if let Some((i, c)) = self.get() {
                match c {
                    'b' => return self.parse_int(start, Radix::Bin, is_bin_digit),
                    'o' => return self.parse_int(start, Radix::Oct, is_oct_digit),
                    'x' => return self.parse_int(start, Radix::Hex, is_hex_digit),
                    _ => {
                        self.put(i, c);
                    }
                }
            }
        }

        let end = self.parse_digits(is_dec_digit)?;
        match self.eat(|c| c == '.') {
            Some((i, _)) => {
                let end = match self.parse_decimal()? {
                    Some(end) => self.parse_exponent()?.unwrap_or(end),
                    None => i + 1,
                };
                Ok((start..end, Token::Float(self.slice(start..end))))
            }
            None => match self.parse_exponent()? {
                Some(end) => Ok((start..end, Token::Float(self.slice(start..end)))),
                None => Ok((start..end, Token::Int(self.slice(start..end), Radix::Dec))),
            },
        }
    }

    fn parse_int(
        &mut self,
        start: usize,
        radix: Radix,
        f: impl FnMut(char) -> bool,
    ) -> Result<(Span, Token<'a>)> {
        let end = self.parse_digits(f)?;
        Ok((start..end, Token::Int(self.slice(start + 2..end), radix)))
    }

    fn parse_digits(&mut self, mut f: impl FnMut(char) -> bool) -> Result<usize> {
        while let Some((i, c)) = self.get() {
            match c {
                '_' => match self.get() {
                    Some((_, c)) if f(c) => {}
                    _ => return Err(Error::invalid_token(i..i + 1, "expect digits after '_'")),
                },
                c if f(c) => {}
                _ => {
                    self.put(i, c);
                    return Ok(i);
                }
            }
        }
        Ok(self.input.len())
    }

    fn parse_decimal(&mut self) -> Result<Option<usize>> {
        match self.get() {
            Some((_, c)) if is_dec_digit(c) => self.parse_digits(is_dec_digit).map(Some),
            Some((i, c)) => {
                self.put(i, c);
                Ok(None)
            }
            None => Ok(None),
        }
    }

    fn parse_exponent(&mut self) -> Result<Option<usize>> {
        match self.eat(|c| c == 'e' || c == 'E') {
            Some((i, c)) => {
                let (i, c) = self.eat(|c| c == '+' || c == '-').unwrap_or((i, c));
                match self.parse_decimal()? {
                    Some(end) => Ok(Some(end)),
                    None => Err(Error::invalid_token(
                        i..i + 1,
                        format!("expect digits after '{c}'"),
                    )),
                }
            }
            None => Ok(None),
        }
    }

    fn check_num_suffix(&mut self) -> Result<()> {
        match self.eat(is_ident_start) {
            Some((i, _)) => Err(Error::invalid_token(
                i..i + 1,
                "invalid suffix after number",
            )),
            None => Ok(()),
        }
    }

    fn parse_ident(&mut self, start: usize) -> Result<(Span, Token<'a>)> {
        let end = match self.skip_while(is_ident_continue) {
            Some((i, _)) => i,
            None => self.input.len(),
        };
        Ok((start..end, Token::Ident(self.slice(start..end))))
    }

    fn parse_punct(&mut self, start: usize, first: char) -> Result<(Span, Token<'a>)> {
        let mut end = start + 1;
        let mut lookahead = |left: Punct, next: char, right: Punct| match self.eat(|c| c == next) {
            Some((i, _)) => {
                end = i + 1;
                left
            }
            None => right,
        };

        use Punct::*;
        let punct = match first {
            ';' => Semi,
            ':' => Colon,
            ',' => Comma,
            '(' => LParen,
            ')' => RParen,
            '{' => LBrace,
            '}' => RBrace,
            '[' => LBracket,
            ']' => RBracket,
            '=' => lookahead(Eq, '=', EqEq),
            '!' => lookahead(Not, '=', NotEq),
            '<' => lookahead(LAngle, '=', LAngleEq),
            '>' => lookahead(RAngle, '=', RAngleEq),
            '+' => lookahead(Plus, '=', PlusEq),
            '-' => lookahead(Minus, '=', MinusEq),
            '*' => lookahead(Star, '=', StarEq),
            '/' => lookahead(Slash, '=', SlashEq),
            '%' => lookahead(Percent, '=', PercentEq),
            _ => return Err(Error::invalid_token(start..start + 1, "")),
        };
        Ok((start..start + 1, Token::Punct(punct)))
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<(Span, Token<'a>)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_while(|c| c.is_whitespace())
            .map(|(i, c)| match c {
                '"' => self.parse_str(i),
                c if is_dec_digit(c) => {
                    let num = self.parse_num(i, c)?;
                    self.check_num_suffix().map(|_| num)
                }
                c if is_ident_start(c) => self.parse_ident(i),
                _ => self.parse_punct(i, c),
            })
    }
}

fn is_bin_digit(c: char) -> bool {
    c == '0' || c == '1'
}

fn is_oct_digit(c: char) -> bool {
    c >= '0' && c <= '7'
}

fn is_dec_digit(c: char) -> bool {
    c.is_ascii_digit()
}

fn is_hex_digit(c: char) -> bool {
    c.is_ascii_hexdigit()
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}
