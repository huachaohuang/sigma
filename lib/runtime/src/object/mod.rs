use std::cell::UnsafeCell;
use std::fmt;
use std::ptr::NonNull;

use sigma_parser::ast::*;

mod bool;
mod null;

use crate::{Error, Result};

#[derive(Clone)]
pub struct Object(RawObject<()>);

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

impl<T> From<RawObject<T>> for Object {
    fn from(raw: RawObject<T>) -> Self {
        Self(raw.cast())
    }
}

impl From<&str> for Object {
    fn from(_: &str) -> Self {
        todo!()
    }
}

impl From<i64> for Object {
    fn from(_: i64) -> Self {
        todo!()
    }
}

impl From<f64> for Object {
    fn from(_: f64) -> Self {
        todo!()
    }
}

impl From<Vec<Object>> for Object {
    fn from(_: Vec<Object>) -> Self {
        todo!()
    }
}

impl From<Vec<(String, Object)>> for Object {
    fn from(_: Vec<(String, Object)>) -> Self {
        todo!()
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

struct RawObject<T>(NonNull<Inner<T>>);

impl<T> RawObject<T> {
    fn new(ty: RawObject<TypeData>, data: T) -> Self {
        let inner = Box::new(Inner { rc: 1, ty, data });
        Self(NonNull::from(Box::leak(inner)))
    }

    fn cast<U>(self) -> RawObject<U> {
        RawObject(self.cast_inner())
    }

    fn cast_inner<U>(&self) -> NonNull<Inner<U>> {
        self.0.cast()
    }

    unsafe fn data<U>(&self) -> &U {
        &self.cast_inner::<U>().as_ref().data
    }
}

impl<T> Clone for RawObject<T> {
    fn clone(&self) -> Self {
        todo!()
    }
}

#[repr(C)]
struct Inner<T> {
    rc: usize,
    ty: RawObject<TypeData>,
    data: T,
}

struct TypeData {
    name: String,

    fmt: FmtFn,
}

thread_local! {
    static TYPE_TYPE: RawObject<TypeData> = RawObject(unsafe {
        NonNull::new_unchecked(TYPE_TYPE_DATA.with(|x| x.get()))
    });

    static TYPE_TYPE_DATA: UnsafeCell<Inner<TypeData>> = UnsafeCell::new(Inner {
        rc: 1,
        ty: TYPE_TYPE.with(|x| x.clone()),
        data: TypeData {
            name: "type".into(),
            fmt: |_, f| write!(f, "type"),
        }
    });
}

type FmtFn = fn(&Object, &mut fmt::Formatter) -> fmt::Result;
