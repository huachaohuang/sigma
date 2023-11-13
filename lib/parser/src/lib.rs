pub mod ast;
use ast::*;

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
    saved: Option<(Span, Token<'a>)>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            lexer: Lexer::new(input),
            saved: None,
        }
    }

    fn take(&mut self) -> Result<(Span, Token<'a>)> {
        match self.saved.take() {
            Some(x) => Ok(x),
            None => self.lexer.next(),
        }
    }

    fn save(&mut self, span: Span, token: Token<'a>) {
        self.saved = Some((span, token));
    }

    fn maybe_punct(&mut self, x: Punct) -> Result<Option<Span>> {
        let (span, token) = self.take()?;
        match token {
            Token::Punct(p) if p == x => Ok(Some(span)),
            _ => {
                self.save(span, token);
                Ok(None)
            }
        }
    }

    fn expect_punct(&mut self, x: Punct) -> Result<Span> {
        let (span, token) = self.take()?;
        match token {
            Token::Punct(p) if p == x => Ok(span),
            _ => {
                self.save(span.clone(), token);
                Err(Error::unexpected_token(span, format!("expect '{x}'")))
            }
        }
    }
}

// Statements
impl<'a> Parser<'a> {
    fn parse_stmt(&mut self, span: Span, token: Token<'a>) -> Result<Stmt<'a>> {
        self.save(span, token);
        self.parse_expr_stmt()
    }

    fn parse_expr_stmt(&mut self) -> Result<Stmt<'a>> {
        let expr = self.parse_expr()?;
        Ok(Stmt::new(expr.span.clone(), StmtKind::Expr(expr)))
    }
}

// Expressions
impl<'a> Parser<'a> {
    fn parse_expr(&mut self) -> Result<Expr<'a>> {
        self.parse_primary_expr()
    }

    fn parse_primary_expr(&mut self) -> Result<Expr<'a>> {
        let mut expr = self.parse_atom_expr()?;
        loop {
            let (_, token) = self.take()?;
            match token {
                Token::Punct(Punct::LParen) => {
                    let (args, paren) =
                        self.parse_terminated_list(Punct::RParen, Self::parse_expr)?;
                    expr = Expr::call(expr.span.start..paren.end, expr, args);
                }
                Token::Punct(Punct::LBracket) => {
                    let index = self.parse_expr()?;
                    let bracket = self.expect_punct(Punct::RBracket)?;
                    expr = Expr::index(expr.span.start..bracket.end, expr, index);
                }
                Token::Punct(Punct::Dot) => {
                    let field = self.parse_field()?;
                    expr = Expr::field(expr, field);
                }
                _ => return Ok(expr),
            }
        }
    }

    fn parse_atom_expr(&mut self) -> Result<Expr<'a>> {
        let (span, token) = self.take()?;
        match token {
            Token::Str(s) => Ok(Expr::lit(span, LitKind::Str(s))),
            Token::Int(s, radix) => Ok(Expr::lit(span, LitKind::Int(s, radix))),
            Token::Float(s) => Ok(Expr::lit(span, LitKind::Float(s))),
            Token::Punct(Punct::LParen) => self.parse_paren_expr(span.start),
            Token::Punct(Punct::LBrace) => self.parse_brace_expr(span.start),
            Token::Punct(Punct::LBracket) => self.parse_bracket_expr(span.start),
            _ => todo!(),
        }
    }

    fn parse_paren_expr(&mut self, start: usize) -> Result<Expr<'a>> {
        let expr = self.parse_expr()?;
        let span = self.expect_punct(Punct::RParen)?;
        Ok(Expr::new(start..span.end, expr.kind))
    }

    fn parse_brace_expr(&mut self, start: usize) -> Result<Expr<'a>> {
        self.parse_terminated_list(Punct::RBrace, Self::parse_field_pair)
            .map(|(list, span)| Expr::hash(start..span.end, list))
    }

    fn parse_field(&mut self) -> Result<Field<'a>> {
        todo!()
    }

    fn parse_field_pair(&mut self) -> Result<(Field<'a>, Expr<'a>)> {
        let field = self.parse_field()?;
        self.expect_punct(Punct::Colon)?;
        let value = self.parse_expr()?;
        Ok((field, value))
    }

    fn parse_bracket_expr(&mut self, start: usize) -> Result<Expr<'a>> {
        self.parse_terminated_list(Punct::RBracket, Self::parse_expr)
            .map(|(list, span)| Expr::list(start..span.end, list))
    }

    fn parse_terminated_list<O>(
        &mut self,
        end: Punct,
        mut f: impl FnMut(&mut Self) -> Result<O>,
    ) -> Result<(Vec<O>, Span)> {
        let mut list = Vec::new();
        loop {
            if let Some(span) = self.maybe_punct(end)? {
                return Ok((list, span));
            }
            list.push(f(self)?);
            if self.maybe_punct(Punct::Comma)?.is_none() {
                break;
            }
        }
        let span = self.expect_punct(end)?;
        Ok((list, span))
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Stmt<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.take() {
            Ok((_, Token::End)) => None,
            Ok((span, token)) => Some(self.parse_stmt(span, token)),
            Err(err) => Some(Err(err)),
        }
    }
}
