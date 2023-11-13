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
}

#[derive(Clone, Debug)]
pub enum ExprKind<'a> {
    Lit(Lit<'a>),
    List(Vec<Expr<'a>>),
    Hash(Vec<(Field<'a>, Expr<'a>)>),
    Call(Box<Expr<'a>>, Vec<Expr<'a>>),
    Index(Box<Expr<'a>>, Box<Expr<'a>>),
    Field(Box<Expr<'a>>, Field<'a>),
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
