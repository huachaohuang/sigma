use sigma_parser::ast::*;

use crate::{Error, Result};

#[derive(Clone, Debug)]
pub struct Object;

impl Object {
    pub(crate) fn index(&self, index: &Object) -> Result<Object> {
        todo!()
    }

    pub(crate) fn set_index(&self, index: &Object, value: Object) -> Result<()> {
        todo!()
    }

    pub(crate) fn field(&self, field: &str) -> Result<Object> {
        todo!()
    }

    pub(crate) fn set_field(&self, field: &str, value: Object) -> Result<()> {
        todo!()
    }

    pub(crate) fn unop(&self, op: UnOp) -> Result<Object> {
        todo!()
    }

    pub(crate) fn binop(&self, op: BinOp, value: &Object) -> Result<Object> {
        todo!()
    }

    pub(crate) fn cmpop(&self, op: CmpOp, value: &Object) -> Result<Object> {
        todo!()
    }
}

impl From<&str> for Object {
    fn from(_: &str) -> Self {
        Object
    }
}

impl From<i64> for Object {
    fn from(_: i64) -> Self {
        Object
    }
}

impl From<f64> for Object {
    fn from(_: f64) -> Self {
        Object
    }
}

impl From<Vec<Object>> for Object {
    fn from(_: Vec<Object>) -> Self {
        Object
    }
}

impl From<Vec<(String, Object)>> for Object {
    fn from(_: Vec<(String, Object)>) -> Self {
        Object
    }
}
