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

#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub enum Radix {
    Bin = 2,
    Oct = 8,
    Dec = 10,
    Hex = 16,
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
            // None => self.lexer.next(),
            None => {
                let x = self.lexer.next();
                println!("{x:?}");
                x
            }
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
                self.save(span.clone(), token.clone());
                Err(token_error(span, token, format!("expect '{}'", x)))
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
                self.save(span.clone(), token.clone());
                Err(token_error(span, token, format!("expect '{x}'")))
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
        let expr = self.parse_lazy_or_expr()?;
        let (span, token) = self.take()?;
        let kind = match token {
            Token::Punct(Punct::Eq) => {
                let value = self.parse_expr()?;
                return Ok(Expr::assign(expr, value));
            }
            Token::Punct(Punct::OrEq) => BinOp::Or,
            Token::Punct(Punct::XorEq) => BinOp::Xor,
            Token::Punct(Punct::AndEq) => BinOp::And,
            Token::Punct(Punct::LShiftEq) => BinOp::Shl,
            Token::Punct(Punct::RShiftEq) => BinOp::Shr,
            Token::Punct(Punct::PlusEq) => BinOp::Add,
            Token::Punct(Punct::MinusEq) => BinOp::Sub,
            Token::Punct(Punct::StarEq) => BinOp::Mul,
            Token::Punct(Punct::SlashEq) => BinOp::Div,
            Token::Punct(Punct::PercentEq) => BinOp::Rem,
            _ => {
                self.save(span, token);
                return Ok(expr);
            }
        };
        let value = self.parse_expr()?;
        Ok(Expr::compound_assign(Spanned::new(span, kind), expr, value))
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

    fn parse_cmp_op(&mut self) -> Result<Option<Spanned<RelOp>>> {
        let (span, token) = self.take()?;
        let op = match token {
            Token::Punct(Punct::EqEq) => RelOp::Eq,
            Token::Punct(Punct::NotEq) => RelOp::Ne,
            Token::Punct(Punct::LAngle) => RelOp::Lt,
            Token::Punct(Punct::LAngleEq) => RelOp::Le,
            Token::Punct(Punct::RAngle) => RelOp::Gt,
            Token::Punct(Punct::RAngleEq) => RelOp::Ge,
            Token::Ident(IN) => RelOp::In,
            Token::Ident(NOT) => {
                self.expect_kw(IN)?;
                RelOp::NotIn
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
            lhs = Expr::relop(op, lhs, rhs);
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
            _ => {
                self.save(span, token);
                return self.parse_primary_expr();
            }
        };
        let expr = self.parse_unary_expr()?;
        Ok(Expr::unop(Spanned::new(span, op), expr))
    }

    fn parse_primary_expr(&mut self) -> Result<Expr<'a>> {
        let mut expr = self.parse_atom_expr()?;
        loop {
            let (span, token) = self.take()?;
            match token {
                Token::Punct(Punct::LBracket) => {
                    let index = self.parse_expr()?;
                    let bracket = self.expect_punct(Punct::RBracket)?;
                    expr = Expr::index(expr.span.start..bracket.end, expr, index);
                }
                Token::Punct(Punct::Dot) => {
                    let field = self.parse_field_name()?;
                    expr = Expr::field(expr, field);
                }
                _ => {
                    self.save(span, token);
                    return Ok(expr);
                }
            }
        }
    }

    fn parse_atom_expr(&mut self) -> Result<Expr<'a>> {
        let (span, token) = self.take()?;
        match token {
            Token::Str(s) => Ok(Expr::lit(span, LitKind::Str(s))),
            Token::Int(s, radix) => Ok(Expr::lit(span, LitKind::Int(s, radix))),
            Token::Float(s) => Ok(Expr::lit(span, LitKind::Float(s))),
            Token::Ident(NULL) => Ok(Expr::lit(span, LitKind::Null)),
            Token::Ident(TRUE) => Ok(Expr::lit(span, LitKind::Bool(true))),
            Token::Ident(FALSE) => Ok(Expr::lit(span, LitKind::Bool(false))),
            Token::Ident(name) => Ok(Expr::name(span, name)),
            Token::Punct(Punct::LParen) => self.parse_paren_expr(span.start),
            Token::Punct(Punct::LBrace) => self.parse_brace_expr(span.start),
            Token::Punct(Punct::LBracket) => self.parse_bracket_expr(span.start),
            _ => Err(token_error(span, token, "expect an expression")),
        }
    }

    fn parse_paren_expr(&mut self, start: usize) -> Result<Expr<'a>> {
        let expr = self.parse_expr()?;
        let span = self.expect_punct(Punct::RParen)?;
        Ok(Expr::new(start..span.end, expr.kind))
    }

    fn parse_field_name(&mut self) -> Result<Field<'a>> {
        let (span, token) = self.take()?;
        let name = match token {
            Token::Str(s) => s,
            Token::Ident(s) => s,
            _ => {
                self.save(span.clone(), token.clone());
                return Err(token_error(span, token, "expect a field name"));
            }
        };
        Ok(Field { span, name })
    }

    fn parse_field_pair(&mut self) -> Result<(Field<'a>, Expr<'a>)> {
        let name = self.parse_field_name()?;
        self.expect_punct(Punct::Colon)?;
        let expr = self.parse_expr()?;
        Ok((name, expr))
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

fn token_error(span: Span, token: Token, message: impl ToString) -> Error {
    if matches!(token, Token::End) {
        Error::incomplete(span, message)
    } else {
        Error::unexpected_token(span, message)
    }
}
