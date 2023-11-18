use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use sigma_parser::ast::*;

mod error;
pub use error::{Error, Result};

mod object;
pub use object::Object;

pub struct Runtime {
    builtin: Builtin,
    closure: Rc<RefCell<Closure>>,
}

impl Runtime {
    fn new() -> Self {
        object::init();
        let builtin = Builtin::new();
        let closure = Rc::new(RefCell::default());
        Self { builtin, closure }
    }

    fn var(&self, name: &str) -> Option<Object> {
        self.closure.borrow().var(name)
    }

    fn set_var(&self, name: impl ToString, value: Object) {
        self.closure.borrow_mut().set_var(name.to_string(), value);
    }
}

impl Runtime {
    pub fn exec(&self, stmt: &Stmt) -> Result<Option<Object>> {
        match &stmt.kind {
            StmtKind::Expr(expr) => self.eval(expr).map(Some),
            StmtKind::Import(name) => self.exec_import(name).map(|_| None),
        }
    }

    fn exec_import(&self, ident: &Ident) -> Result<()> {
        if let Some(module) = self.builtin.modules.get(ident.name).cloned() {
            self.set_var(ident.name, module);
            Ok(())
        } else {
            Err(Error::with_span(
                ident.span.clone(),
                format!("module '{}' is not defined", ident.name),
            ))
        }
    }

    fn eval(&self, expr: &Expr) -> Result<Object> {
        match &expr.kind {
            ExprKind::Lit(lit) => self.eval_lit(lit),
            ExprKind::Name(name) => self.eval_name(name),
            ExprKind::List(list) => self.eval_list(list),
            ExprKind::Hash(hash) => self.eval_hash(hash),
            ExprKind::Index(expr, index) => self.eval_index(expr, index),
            ExprKind::Field(expr, field) => self.eval_field(expr, field),
            ExprKind::UnOp(op, expr) => self.eval_unop(op, expr),
            ExprKind::BinOp(op, lhs, rhs) => self.eval_binop(op, lhs, rhs),
            ExprKind::RelOp(op, lhs, rhs) => self.eval_relop(op, lhs, rhs),
            ExprKind::BoolOp(op, lhs, rhs) => self.eval_boolop(op, lhs, rhs),
            ExprKind::Assign(lhs, rhs) => self.eval_assign(lhs, rhs),
            ExprKind::CompoundAssign(op, lhs, rhs) => self.eval_compound_assign(op, lhs, rhs),
        }
        .map_err(|mut e| {
            if e.span.is_empty() {
                e.span = expr.span.clone();
            }
            e
        })
    }

    fn eval_lit(&self, lit: &Lit) -> Result<Object> {
        match lit.kind {
            LitKind::Null => Ok(self.builtin.null.clone()),
            LitKind::Bool(true) => Ok(self.builtin.true_.clone()),
            LitKind::Bool(false) => Ok(self.builtin.false_.clone()),
            LitKind::Str(s) => Ok(s.into()),
            LitKind::Int(s, radix) => i64::from_str_radix(s, radix as u32)
                .map(Into::into)
                .map_err(Error::new),
            LitKind::Float(s) => s.parse::<f64>().map(Into::into).map_err(Error::new),
        }
    }

    fn eval_name(&self, ident: &Ident) -> Result<Object> {
        self.var(ident.name).ok_or_else(|| {
            Error::with_span(
                ident.span.clone(),
                format!("name '{}' is not defined", ident.name),
            )
        })
    }

    fn eval_list(&self, list: &[Expr]) -> Result<Object> {
        list.iter()
            .map(|expr| self.eval(expr))
            .collect::<Result<Vec<_>>>()
            .map(|list| list.into())
    }

    fn eval_hash(&self, hash: &[(Field, Expr)]) -> Result<Object> {
        hash.iter()
            .map(|(field, expr)| self.eval(expr).map(|value| (field.name.to_owned(), value)))
            .collect::<Result<Vec<_>>>()
            .map(|hash| hash.into())
    }

    fn eval_index(&self, expr: &Expr, index: &Expr) -> Result<Object> {
        let this = self.eval(expr)?;
        let value = self.eval(index)?;
        this.index(&value)
    }

    fn eval_field(&self, expr: &Expr, field: &Field) -> Result<Object> {
        let this = self.eval(expr)?;
        this.field(field.name)
    }

    fn eval_unop(&self, op: &Spanned<UnOp>, expr: &Expr) -> Result<Object> {
        let this = self.eval(expr)?;
        this.unop(op.kind)
    }

    fn eval_binop(&self, op: &Spanned<BinOp>, lhs: &Expr, rhs: &Expr) -> Result<Object> {
        let this = self.eval(lhs)?;
        let other = self.eval(rhs)?;
        this.binop(op.kind, &other)
    }

    fn eval_relop(&self, op: &Spanned<RelOp>, lhs: &Expr, rhs: &Expr) -> Result<Object> {
        let this = self.eval(lhs)?;
        let other = self.eval(rhs)?;
        this.relop(op.kind, &other)
    }

    fn eval_boolop(&self, op: &Spanned<BoolOp>, lhs: &Expr, rhs: &Expr) -> Result<Object> {
        let this = self.eval(lhs)?;
        let is_true = this.as_bool().ok_or_else(|| {
            Error::with_span(
                lhs.span.clone(),
                "left-hand side should be a boolean expression",
            )
        })?;
        if is_true {
            match op.kind {
                BoolOp::Or => Ok(this),
                BoolOp::And => self.eval(rhs),
            }
        } else {
            match op.kind {
                BoolOp::Or => self.eval(rhs),
                BoolOp::And => Ok(this),
            }
        }
    }

    fn eval_assign(&self, lhs: &Expr, rhs: &Expr) -> Result<Object> {
        let value = self.eval(rhs)?;
        match &lhs.kind {
            ExprKind::Name(ident) => {
                self.set_var(ident.name, value.clone());
                Ok(value)
            }
            ExprKind::Index(expr, index) => {
                let mut this = self.eval(expr)?;
                let index = self.eval(index)?;
                this.set_index(&index, value.clone())?;
                Ok(value)
            }
            ExprKind::Field(expr, field) => {
                let mut this = self.eval(expr)?;
                this.set_field(field.name, value.clone())?;
                Ok(value)
            }
            _ => Err(Error::with_span(
                lhs.span.clone(),
                "invalid target for assignment",
            )),
        }
    }

    fn eval_compound_assign(&self, op: &Spanned<BinOp>, lhs: &Expr, rhs: &Expr) -> Result<Object> {
        let value = self.eval(rhs)?;
        match &lhs.kind {
            ExprKind::Name(ident) => {
                let old_value = self.eval_name(ident)?;
                let new_value = old_value.binop(op.kind, &value)?;
                self.set_var(ident.name, new_value.clone());
                Ok(new_value)
            }
            ExprKind::Index(expr, index) => {
                let mut this = self.eval(expr)?;
                let index = self.eval(index)?;
                let old_value = this.index(&index)?;
                let new_value = old_value.binop(op.kind, &value)?;
                this.set_index(&index, new_value.clone())?;
                Ok(new_value)
            }
            ExprKind::Field(expr, field) => {
                let mut this = self.eval(expr)?;
                let old_value = this.field(field.name)?;
                let new_value = old_value.binop(op.kind, &value)?;
                this.set_field(field.name, new_value.clone())?;
                Ok(new_value)
            }
            _ => Err(Error::with_span(
                lhs.span.clone(),
                "invalid target for assignment",
            )),
        }
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

struct Builtin {
    null: Object,
    true_: Object,
    false_: Object,
    modules: HashMap<String, Object>,
}

impl Builtin {
    fn new() -> Self {
        Self {
            null: ().into(),
            true_: true.into(),
            false_: false.into(),
            modules: HashMap::new(),
        }
    }
}

type Vars = HashMap<String, Object>;

#[derive(Default)]
struct Closure {
    vars: Vars,
    outer: Option<Rc<Closure>>,
}

impl Closure {
    fn var(&self, name: &str) -> Option<Object> {
        self.vars
            .get(name)
            .cloned()
            .or_else(|| self.outer.as_ref().and_then(|outer| outer.var(name)))
    }

    fn set_var(&mut self, name: String, value: Object) {
        self.vars.insert(name, value);
    }
}
