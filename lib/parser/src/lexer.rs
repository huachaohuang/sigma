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
            Some((i, c)) => {
                self.save(i, c);
                i
            }
            None => self.input.len(),
        };
        Ok((start..end, Token::Ident(self.slice(start..end))))
    }

    fn parse_punct(&mut self, start: usize, first: char) -> Result<(Span, Token<'a>)> {
        use Punct::*;
        let (punct, count) = match first {
            ';' => (Semi, 1),
            ':' => (Colon, 1),
            ',' => (Comma, 1),
            '.' => self.parse_punct_1(Dot, '.', DotDot),
            '(' => (LParen, 1),
            ')' => (RParen, 1),
            '{' => (LBrace, 1),
            '}' => (RBrace, 1),
            '[' => (LBracket, 1),
            ']' => (RBracket, 1),
            '=' => self.parse_punct_1(Eq, '=', EqEq),
            '!' => self.parse_punct_1(Not, '=', NotEq),
            '+' => self.parse_punct_1(Plus, '=', PlusEq),
            '-' => self.parse_punct_1(Minus, '=', MinusEq),
            '*' => self.parse_punct_1(Star, '=', StarEq),
            '/' => self.parse_punct_1(Slash, '=', SlashEq),
            '%' => self.parse_punct_1(Percent, '=', PercentEq),
            '|' => self.parse_punct_2(Or, '=', OrEq, '|', OrOr),
            '^' => self.parse_punct_1(Xor, '=', XorEq),
            '&' => self.parse_punct_2(And, '=', AndEq, '&', AndAnd),
            '<' => self.parse_punct_3(LAngle, '=', LAngleEq, '<', LShift, '=', LShiftEq),
            '>' => self.parse_punct_3(RAngle, '=', RAngleEq, '>', RShift, '=', RShiftEq),
            _ => return Err(Error::invalid_token(start..start + 1, "")),
        };
        Ok((start..start + count, Token::Punct(punct)))
    }

    fn parse_punct_1(&mut self, default: Punct, x: char, matched: Punct) -> (Punct, usize) {
        match self.take() {
            Some((_, c)) if c == x => (matched, 2),
            Some((i, c)) => {
                self.save(i, c);
                (default, 1)
            }
            None => (default, 1),
        }
    }

    fn parse_punct_2(
        &mut self,
        default: Punct,
        x1: char,
        matched1: Punct,
        x2: char,
        matched2: Punct,
    ) -> (Punct, usize) {
        match self.take() {
            Some((_, c)) if c == x1 => (matched1, 2),
            Some((_, c)) if c == x2 => (matched2, 2),
            Some((i, c)) => {
                self.save(i, c);
                (default, 1)
            }
            None => (default, 1),
        }
    }

    fn parse_punct_3(
        &mut self,
        default: Punct,
        x1: char,
        matched1: Punct,
        x2: char,
        matched2: Punct,
        x3: char,
        matched3: Punct,
    ) -> (Punct, usize) {
        match self.take() {
            Some((_, c)) if c == x1 => (matched1, 2),
            Some((_, c)) if c == x2 => {
                let (punct, count) = self.parse_punct_1(matched2, x3, matched3);
                (punct, count + 1)
            }
            Some((i, c)) => {
                self.save(i, c);
                (default, 1)
            }
            None => (default, 1),
        }
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
