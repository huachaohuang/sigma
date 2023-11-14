use sigma_parser::ast::*;
use sigma_parser::Span;

mod object;
pub use object::Object;

#[derive(Debug)]
pub struct Error {
    span: Span,
    message: String,
}

impl Error {
    fn new(span: Span, message: impl ToString) -> Self {
        Self {
            span,
            message: message.to_string(),
        }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Default)]
pub struct Runtime;

impl Runtime {
    pub fn exec(&self, stmt: &Stmt) -> Result<Option<Object>> {
        todo!()
    }
}

struct Core {
    global: Global,
}

impl Core {
    fn eval(&mut self, expr: &Expr) -> Result<Object> {
        match &expr.kind {
            ExprKind::Lit(lit) => self.eval_lit(lit),
            ExprKind::List(list) => self.eval_list(list),
            ExprKind::Hash(hash) => self.eval_hash(hash),
            ExprKind::Index(expr, index) => self.eval_index(expr, index),
            ExprKind::Field(expr, field) => self.eval_field(expr, field),
            ExprKind::UnOp(op, expr) => self.eval_unop(op, expr),
            ExprKind::BinOp(op, lhs, rhs) => self.eval_binop(op, lhs, rhs),
            ExprKind::CmpOp(op, lhs, rhs) => self.eval_cmpop(op, lhs, rhs),
            ExprKind::BoolOp(op, lhs, rhs) => self.eval_boolop(op, lhs, rhs),
            ExprKind::Assign(lhs, rhs) => self.eval_assign(lhs, rhs),
            ExprKind::CompoundAssign(op, lhs, rhs) => self.eval_compound_assign(op, lhs, rhs),
            _ => Err(Error::new(expr.span.clone(), "unsupported expression")),
        }
    }

    fn eval_lit(&mut self, lit: &Lit) -> Result<Object> {
        match lit.kind {
            LitKind::Null => Ok(self.global.null.clone()),
            LitKind::Bool(true) => Ok(self.global.true_.clone()),
            LitKind::Bool(false) => Ok(self.global.false_.clone()),
            LitKind::Str(s) => Ok(s.into()),
            LitKind::Int(s, radix) => i64::from_str_radix(s, radix as u32)
                .map(Into::into)
                .map_err(|e| Error::new(lit.span.clone(), e)),
            LitKind::Float(s) => s
                .parse::<f64>()
                .map(Into::into)
                .map_err(|e| Error::new(lit.span.clone(), e)),
        }
    }

    fn eval_list(&mut self, list: &[Expr]) -> Result<Object> {
        list.iter()
            .map(|expr| self.eval(expr))
            .collect::<Result<Vec<_>>>()
            .map(|list| list.into())
    }

    fn eval_hash(&mut self, hash: &[(Field, Expr)]) -> Result<Object> {
        hash.iter()
            .map(|(field, expr)| self.eval(expr).map(|value| (field.name.to_owned(), value)))
            .collect::<Result<Vec<_>>>()
            .map(|hash| hash.into())
    }

    fn eval_index(&mut self, expr: &Expr, index: &Expr) -> Result<Object> {
        let ob = self.eval(expr)?;
        let index = self.eval(index)?;
        todo!()
    }

    fn eval_field(&mut self, expr: &Expr, field: &Field) -> Result<Object> {
        let ob = self.eval(expr)?;
        todo!()
    }

    fn eval_unop(&mut self, op: &Spanned<UnOp>, expr: &Expr) -> Result<Object> {
        let ob = self.eval(expr)?;
        todo!()
    }

    fn eval_binop(&mut self, op: &Spanned<BinOp>, lhs: &Expr, rhs: &Expr) -> Result<Object> {
        let lv = self.eval(lhs)?;
        let rv = self.eval(rhs)?;
        todo!()
    }

    fn eval_cmpop(&mut self, op: &Spanned<CmpOp>, lhs: &Expr, rhs: &Expr) -> Result<Object> {
        let lv = self.eval(lhs)?;
        let rv = self.eval(rhs)?;
        todo!()
    }

    fn eval_boolop(&mut self, op: &Spanned<BoolOp>, lhs: &Expr, rhs: &Expr) -> Result<Object> {
        let lv = self.eval(lhs)?;
        todo!()
    }

    fn eval_assign(&mut self, lhs: &Expr, rhs: &Expr) -> Result<Object> {
        let value = self.eval(rhs)?;
        todo!()
    }

    fn eval_compound_assign(
        &mut self,
        op: &Spanned<BinOp>,
        lhs: &Expr,
        rhs: &Expr,
    ) -> Result<Object> {
        let value = self.eval(rhs)?;
        todo!()
    }
}

struct Global {
    null: Object,
    true_: Object,
    false_: Object,
}
