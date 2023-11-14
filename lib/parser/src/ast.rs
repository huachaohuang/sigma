use crate::{Radix, Span};

#[derive(Clone, Debug)]
pub struct Stmt<'a> {
    pub span: Span,
    pub kind: StmtKind<'a>,
}

impl<'a> Stmt<'a> {
    pub(crate) fn new(span: Span, kind: StmtKind<'a>) -> Self {
        Self { span, kind }
    }
}

#[derive(Clone, Debug)]
pub enum StmtKind<'a> {
    Expr(Expr<'a>),
}

#[derive(Clone, Debug)]
pub struct Expr<'a> {
    pub span: Span,
    pub kind: ExprKind<'a>,
}

impl<'a> Expr<'a> {
    pub(crate) fn new(span: Span, kind: ExprKind<'a>) -> Self {
        Self { span, kind }
    }

    pub(crate) fn lit(span: Span, kind: LitKind<'a>) -> Self {
        Self::new(span.clone(), ExprKind::Lit(Lit { span, kind }))
    }

    pub(crate) fn name(span: Span, name: &'a str) -> Self {
        Self::new(span.clone(), ExprKind::Name(Ident { span, name }))
    }

    pub(crate) fn list(span: Span, list: Vec<Expr<'a>>) -> Self {
        Self::new(span, ExprKind::List(list))
    }

    pub(crate) fn hash(span: Span, list: Vec<(Field<'a>, Expr<'a>)>) -> Self {
        Self::new(span, ExprKind::Hash(list))
    }

    pub(crate) fn call(span: Span, expr: Expr<'a>, args: Vec<Expr<'a>>) -> Self {
        Self::new(span, ExprKind::Call(expr.into(), args))
    }

    pub(crate) fn index(span: Span, expr: Expr<'a>, index: Expr<'a>) -> Self {
        Self::new(span, ExprKind::Index(expr.into(), index.into()))
    }

    pub(crate) fn field(expr: Expr<'a>, field: Field<'a>) -> Self {
        let span = expr.span.start..field.span.end;
        Self::new(span, ExprKind::Field(expr.into(), field))
    }

    pub(crate) fn unop(op: Spanned<UnOp>, expr: Expr<'a>) -> Self {
        let span = op.span.start..expr.span.end;
        Self::new(span, ExprKind::UnOp(op, expr.into()))
    }

    pub(crate) fn binop(op: Spanned<BinOp>, lhs: Expr<'a>, rhs: Expr<'a>) -> Self {
        let span = lhs.span.start..rhs.span.end;
        Self::new(span, ExprKind::BinOp(op, lhs.into(), rhs.into()))
    }

    pub(crate) fn cmpop(op: Spanned<CmpOp>, lhs: Expr<'a>, rhs: Expr<'a>) -> Self {
        let span = lhs.span.start..rhs.span.end;
        Self::new(span, ExprKind::CmpOp(op, lhs.into(), rhs.into()))
    }

    pub(crate) fn boolop(op: Spanned<BoolOp>, lhs: Expr<'a>, rhs: Expr<'a>) -> Self {
        let span = lhs.span.start..rhs.span.end;
        Self::new(span, ExprKind::BoolOp(op, lhs.into(), rhs.into()))
    }
}

#[derive(Clone, Debug)]
pub enum ExprKind<'a> {
    Lit(Lit<'a>),
    Name(Ident<'a>),
    List(Vec<Expr<'a>>),
    Hash(Vec<(Field<'a>, Expr<'a>)>),
    Call(Box<Expr<'a>>, Vec<Expr<'a>>),
    Index(Box<Expr<'a>>, Box<Expr<'a>>),
    Field(Box<Expr<'a>>, Field<'a>),
    UnOp(Spanned<UnOp>, Box<Expr<'a>>),
    BinOp(Spanned<BinOp>, Box<Expr<'a>>, Box<Expr<'a>>),
    CmpOp(Spanned<CmpOp>, Box<Expr<'a>>, Box<Expr<'a>>),
    BoolOp(Spanned<BoolOp>, Box<Expr<'a>>, Box<Expr<'a>>),
}

#[derive(Clone, Debug)]
pub struct Lit<'a> {
    pub span: Span,
    pub kind: LitKind<'a>,
}

#[derive(Clone, Debug)]
pub enum LitKind<'a> {
    Str(&'a str),
    Int(&'a str, Radix),
    Float(&'a str),
}

#[derive(Clone, Debug)]
pub struct Ident<'a> {
    pub span: Span,
    pub name: &'a str,
}

#[derive(Clone, Debug)]
pub struct Field<'a> {
    pub span: Span,
    pub name: &'a str,
}

#[derive(Clone, Debug)]
pub struct Spanned<T> {
    pub span: Span,
    pub kind: T,
}

impl<T> Spanned<T> {
    pub(crate) fn new(span: Span, kind: T) -> Self {
        Self { span, kind }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnOp {
    Not,
    Neg,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Or,
    Xor,
    And,
    Shl,
    Shr,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    In,
    NotIn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoolOp {
    Or,
    And,
}
