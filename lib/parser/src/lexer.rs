use std::str::CharIndices;

use crate::token::*;
use crate::{Error, Radix, Result, Span};

pub(crate) struct Lexer<'a> {
    input: &'a str,
    chars: CharIndices<'a>,
    saved: Option<(usize, char)>,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.char_indices(),
            saved: None,
        }
    }

    pub(crate) fn next(&mut self) -> Result<(Span, Token<'a>)> {
        match self.skip_while(|c| c.is_whitespace()) {
            Some((i, c)) => match c {
                '"' => self.parse_str(i),
                c if is_dec_digit(c) => {
                    let num = self.parse_num(i, c)?;
                    self.check_num_suffix().map(|_| num)
                }
                c if is_ident_start(c) => self.parse_ident(i),
                _ => self.parse_punct(i, c),
            },
            None => {
                let len = self.input.len();
                Ok((len..len, Token::End))
            }
        }
    }

    fn take(&mut self) -> Option<(usize, char)> {
        self.saved.take().or_else(|| self.chars.next())
    }

    fn save(&mut self, i: usize, c: char) {
        self.saved = Some((i, c));
    }

    fn slice(&self, span: Span) -> &'a str {
        // SAFETY: `span` is always valid for internal use.
        unsafe { self.input.get_unchecked(span) }
    }

    fn take_if(&mut self, f: impl FnOnce(char) -> bool) -> Option<(usize, char)> {
        self.take().and_then(|(i, c)| {
            if f(c) {
                Some((i, c))
            } else {
                self.save(i, c);
                None
            }
        })
    }

    fn skip_while(&mut self, mut f: impl FnMut(char) -> bool) -> Option<(usize, char)> {
        while let Some((i, c)) = self.take() {
            if !f(c) {
                return Some((i, c));
            }
        }
        None
    }

    fn parse_str(&mut self, start: usize) -> Result<(Span, Token<'a>)> {
        while let Some((i, c)) = self.take() {
            match c {
                '"' => return Ok((start..i + 1, Token::Str(self.slice(start + 1..i)))),
                '\\' => {
                    let _ = self.take();
                }
                _ => {}
            }
        }
        Err(Error::invalid_token(
            start..self.input.len(),
            "unterminated string literal",
        ))
    }

    fn parse_num(&mut self, start: usize, first: char) -> Result<(Span, Token<'a>)> {
        if first == '0' {
            if let Some((i, c)) = self.take() {
                match c {
                    'b' => return self.parse_int(start, Radix::Bin, is_bin_digit),
                    'o' => return self.parse_int(start, Radix::Oct, is_oct_digit),
                    'x' => return self.parse_int(start, Radix::Hex, is_hex_digit),
                    _ => {
                        self.save(i, c);
                    }
                }
            }
        }

        let end = self.parse_digits(is_dec_digit)?;
        match self.take_if(|c| c == '.') {
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
        is_digit: impl Fn(char) -> bool,
    ) -> Result<(Span, Token<'a>)> {
        let end = self.parse_digits(is_digit)?;
        if end == start + 2 {
            return Err(Error::invalid_token(
                start..end,
                format!("expect digits after '{}'", self.slice(start..end)),
            ));
        }
        Ok((start..end, Token::Int(self.slice(start + 2..end), radix)))
    }

    fn parse_digits(&mut self, is_digit: impl Fn(char) -> bool) -> Result<usize> {
        while let Some((i, c)) = self.take() {
            match c {
                '_' => {
                    if self.take_if(&is_digit).is_none() {
                        return Err(Error::invalid_token(i..i + 1, "expect digits after '_'"));
                    }
                }
                c if is_digit(c) => {}
                _ => {
                    self.save(i, c);
                    return Ok(i);
                }
            }
        }
        Ok(self.input.len())
    }

    fn parse_decimal(&mut self) -> Result<Option<usize>> {
        match self.take_if(is_dec_digit) {
            Some(_) => self.parse_digits(is_dec_digit).map(Some),
            None => Ok(None),
        }
    }

    fn parse_exponent(&mut self) -> Result<Option<usize>> {
        match self.take_if(|c| c == 'e' || c == 'E') {
            Some((i, c)) => {
                let (i, c) = self.take_if(|c| c == '+' || c == '-').unwrap_or((i, c));
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
        match self.take_if(is_ident_start) {
            Some((i, _)) => Err(Error::invalid_token(
                i..i + 1,
                "unexpected suffix after number literal",
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
        let mut lookahead = |next, matched, default| match self.take_if(|c| c == next) {
            Some((i, _)) => {
                end = i + 1;
                matched
            }
            None => default,
        };

        use Punct::*;
        let punct = match first {
            ';' => Semi,
            ':' => Colon,
            ',' => Comma,
            '.' => lookahead('.', DotDot, Dot),
            '(' => LParen,
            ')' => RParen,
            '{' => LBrace,
            '}' => RBrace,
            '[' => LBracket,
            ']' => RBracket,
            '=' => lookahead('=', EqEq, Eq),
            '!' => lookahead('=', NotEq, Not),
            '<' => lookahead('=', LAngleEq, LAngle),
            '>' => lookahead('=', RAngleEq, RAngle),
            '+' => lookahead('=', PlusEq, Plus),
            '-' => lookahead('=', MinusEq, Minus),
            '*' => lookahead('=', StarEq, Star),
            '/' => lookahead('=', SlashEq, Slash),
            '%' => lookahead('=', PercentEq, Percent),
            _ => return Err(Error::invalid_token(start..start + 1, "")),
        };
        Ok((start..start + 1, Token::Punct(punct)))
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
