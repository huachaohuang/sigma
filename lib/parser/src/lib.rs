pub mod ast;
use ast::*;

mod error;
pub use error::{Error, Result};

mod lexer;
use lexer::Lexer;

mod token;
use token::*;

mod keyword;
use keyword::*;

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

    fn expect_kw(&mut self, x: &str) -> Result<Span> {
        let (span, token) = self.take()?;
        match token {
            Token::Ident(s) if s == x => Ok(span),
            _ => {
                self.save(span.clone(), token);
                Err(Error::unexpected_token(span, format!("expect '{}'", x)))
            }
        }
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
        self.parse_lazy_or_expr()
    }

    fn parse_lazy_or_expr(&mut self) -> Result<Expr<'a>> {
        let mut lhs = self.parse_lazy_and_expr()?;
        while let Some(span) = self.maybe_punct(Punct::OrOr)? {
            let rhs = self.parse_lazy_and_expr()?;
            lhs = Expr::boolop(Spanned::new(span, BoolOp::Or), lhs, rhs);
        }
        Ok(lhs)
    }

    fn parse_lazy_and_expr(&mut self) -> Result<Expr<'a>> {
        let mut lhs = self.parse_cmp_expr()?;
        while let Some(span) = self.maybe_punct(Punct::AndAnd)? {
            let rhs = self.parse_cmp_expr()?;
            lhs = Expr::boolop(Spanned::new(span, BoolOp::And), lhs, rhs);
        }
        Ok(lhs)
    }

    fn parse_cmp_op(&mut self) -> Result<Option<Spanned<CmpOp>>> {
        let (span, token) = self.take()?;
        let op = match token {
            Token::Punct(Punct::EqEq) => CmpOp::Eq,
            Token::Punct(Punct::NotEq) => CmpOp::Ne,
            Token::Punct(Punct::LAngle) => CmpOp::Lt,
            Token::Punct(Punct::LAngleEq) => CmpOp::Le,
            Token::Punct(Punct::RAngle) => CmpOp::Gt,
            Token::Punct(Punct::RAngleEq) => CmpOp::Ge,
            Token::Ident(IN) => CmpOp::In,
            Token::Ident(NOT) => {
                self.expect_kw(IN)?;
                CmpOp::NotIn
            }
            _ => {
                self.save(span, token);
                return Ok(None);
            }
        };
        Ok(Some(Spanned::new(span, op)))
    }

    fn parse_cmp_expr(&mut self) -> Result<Expr<'a>> {
        let mut lhs = self.parse_or_expr()?;
        while let Some(op) = self.parse_cmp_op()? {
            let rhs = self.parse_or_expr()?;
            lhs = Expr::cmpop(op, lhs, rhs);
        }
        Ok(lhs)
    }

    fn parse_or_expr(&mut self) -> Result<Expr<'a>> {
        let mut lhs = self.parse_xor_expr()?;
        while let Some(span) = self.maybe_punct(Punct::Or)? {
            let rhs = self.parse_xor_expr()?;
            lhs = Expr::binop(Spanned::new(span, BinOp::Or), lhs, rhs);
        }
        Ok(lhs)
    }

    fn parse_xor_expr(&mut self) -> Result<Expr<'a>> {
        let mut lhs = self.parse_and_expr()?;
        while let Some(span) = self.maybe_punct(Punct::Xor)? {
            let rhs = self.parse_and_expr()?;
            lhs = Expr::binop(Spanned::new(span, BinOp::Xor), lhs, rhs);
        }
        Ok(lhs)
    }

    fn parse_and_expr(&mut self) -> Result<Expr<'a>> {
        let mut lhs = self.parse_shift_expr()?;
        while let Some(span) = self.maybe_punct(Punct::And)? {
            let rhs = self.parse_shift_expr()?;
            lhs = Expr::binop(Spanned::new(span, BinOp::And), lhs, rhs);
        }
        Ok(lhs)
    }

    fn parse_shift_op(&mut self) -> Result<Option<Spanned<BinOp>>> {
        let (span, token) = self.take()?;
        let kind = match token {
            Token::Punct(Punct::LShift) => BinOp::Shl,
            Token::Punct(Punct::RShift) => BinOp::Shr,
            _ => {
                self.save(span, token);
                return Ok(None);
            }
        };
        Ok(Some(Spanned::new(span, kind)))
    }

    fn parse_shift_expr(&mut self) -> Result<Expr<'a>> {
        let mut lhs = self.parse_add_expr()?;
        while let Some(op) = self.parse_shift_op()? {
            let rhs = self.parse_add_expr()?;
            lhs = Expr::binop(op, lhs, rhs);
        }
        Ok(lhs)
    }

    fn parse_add_op(&mut self) -> Result<Option<Spanned<BinOp>>> {
        let (span, token) = self.take()?;
        let kind = match token {
            Token::Punct(Punct::Plus) => BinOp::Add,
            Token::Punct(Punct::Minus) => BinOp::Sub,
            _ => {
                self.save(span, token);
                return Ok(None);
            }
        };
        Ok(Some(Spanned::new(span, kind)))
    }

    fn parse_add_expr(&mut self) -> Result<Expr<'a>> {
        let mut lhs = self.parse_mul_expr()?;
        while let Some(op) = self.parse_add_op()? {
            let rhs = self.parse_mul_expr()?;
            lhs = Expr::binop(op, lhs, rhs);
        }
        Ok(lhs)
    }

    fn parse_mul_op(&mut self) -> Result<Option<Spanned<BinOp>>> {
        let (span, token) = self.take()?;
        let kind = match token {
            Token::Punct(Punct::Star) => BinOp::Mul,
            Token::Punct(Punct::Slash) => BinOp::Div,
            Token::Punct(Punct::Percent) => BinOp::Rem,
            _ => {
                self.save(span, token);
                return Ok(None);
            }
        };
        Ok(Some(Spanned::new(span, kind)))
    }

    fn parse_mul_expr(&mut self) -> Result<Expr<'a>> {
        let mut lhs = self.parse_unary_expr()?;
        while let Some(op) = self.parse_mul_op()? {
            let rhs = self.parse_unary_expr()?;
            lhs = Expr::binop(op, lhs, rhs);
        }
        Ok(lhs)
    }

    fn parse_unary_expr(&mut self) -> Result<Expr<'a>> {
        let (span, token) = self.take()?;
        let op = match token {
            Token::Punct(Punct::Not) => UnOp::Not,
            Token::Punct(Punct::Minus) => UnOp::Neg,
            _ => return self.parse_primary_expr(),
        };
        let expr = self.parse_unary_expr()?;
        Ok(Expr::unop(Spanned::new(span, op), expr))
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
            Token::Ident(name) => Ok(Expr::name(span, name)),
            Token::Punct(Punct::LParen) => self.parse_paren_expr(span.start),
            Token::Punct(Punct::LBrace) => self.parse_brace_expr(span.start),
            Token::Punct(Punct::LBracket) => self.parse_bracket_expr(span.start),
            _ => Err(Error::unexpected_token(span, "expect an expression")),
        }
    }

    fn parse_paren_expr(&mut self, start: usize) -> Result<Expr<'a>> {
        let expr = self.parse_expr()?;
        let span = self.expect_punct(Punct::RParen)?;
        Ok(Expr::new(start..span.end, expr.kind))
    }

    fn parse_field(&mut self) -> Result<Field<'a>> {
        let (span, token) = self.take()?;
        let name = match token {
            Token::Str(s) => s,
            Token::Ident(s) => s,
            _ => return Err(Error::unexpected_token(span, "expect a field name")),
        };
        Ok(Field { span, name })
    }

    fn parse_field_pair(&mut self) -> Result<(Field<'a>, Expr<'a>)> {
        let field = self.parse_field()?;
        self.expect_punct(Punct::Colon)?;
        let value = self.parse_expr()?;
        Ok((field, value))
    }

    fn parse_brace_expr(&mut self, start: usize) -> Result<Expr<'a>> {
        self.parse_terminated_list(Punct::RBrace, Self::parse_field_pair)
            .map(|(list, span)| Expr::hash(start..span.end, list))
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
