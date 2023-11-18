use std::cell::{Cell, UnsafeCell};
use std::cmp::Ordering;
use std::fmt;
use std::ptr::NonNull;

use sigma_parser::ast::*;

mod bool;
mod f64;
mod hash;
mod i64;
mod list;
mod null;
mod str;

use crate::{Error, Result};

#[derive(Clone)]
pub struct Object(RawObject<()>);

impl Object {
    fn type_name(&self) -> &str {
        &self.0.type_data().name
    }

    pub(crate) fn index(&self, index: &Object) -> Result<Object> {
        (self.0.type_data().index)(self, index)
    }

    pub(crate) fn set_index(&mut self, index: &Object, value: Object) -> Result<()> {
        (self.0.type_data().set_index)(self, index, value)
    }

    pub(crate) fn field(&self, field: &str) -> Result<Object> {
        (self.0.type_data().field)(self, field)
    }

    pub(crate) fn set_field(&mut self, field: &str, value: Object) -> Result<()> {
        (self.0.type_data().set_field)(self, field, value)
    }

    pub(crate) fn unop(&self, op: UnOp) -> Result<Object> {
        let arithmetic = &self.0.type_data().arithmetic;
        match op {
            UnOp::Not => (arithmetic.not)(self),
            UnOp::Neg => (arithmetic.neg)(self),
        }
    }

    pub(crate) fn binop(&self, op: BinOp, other: &Object) -> Result<Object> {
        let arithmetic = &self.0.type_data().arithmetic;
        match op {
            BinOp::Or => (arithmetic.or)(self, other),
            BinOp::Xor => (arithmetic.xor)(self, other),
            BinOp::And => (arithmetic.and)(self, other),
            BinOp::Shl => (arithmetic.shl)(self, other),
            BinOp::Shr => (arithmetic.shr)(self, other),
            BinOp::Add => (arithmetic.add)(self, other),
            BinOp::Sub => (arithmetic.sub)(self, other),
            BinOp::Mul => (arithmetic.mul)(self, other),
            BinOp::Div => (arithmetic.div)(self, other),
            BinOp::Rem => (arithmetic.rem)(self, other),
        }
    }

    pub(crate) fn relop(&self, op: RelOp, other: &Object) -> Result<Object> {
        let value = match op {
            RelOp::Eq => self.compare(other)? == Ordering::Equal,
            RelOp::Ne => self.compare(other)? != Ordering::Equal,
            RelOp::Lt => self.compare(other)? == Ordering::Less,
            RelOp::Le => self.compare(other)? != Ordering::Greater,
            RelOp::Gt => self.compare(other)? == Ordering::Greater,
            RelOp::Ge => self.compare(other)? != Ordering::Less,
            RelOp::In => self.contains(other)?,
            RelOp::NotIn => !self.contains(other)?,
        };
        Ok(value.into())
    }

    fn compare(&self, other: &Object) -> Result<Ordering> {
        (self.0.type_data().compare)(self, other)
    }

    fn contains(&self, other: &Object) -> Result<bool> {
        (self.0.type_data().contains)(self, other)
    }
}

impl Drop for Object {
    fn drop(&mut self) {
        self.0.unref();
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.0.type_data().format)(self, f)
    }
}

struct RawObject<T>(NonNull<Inner<T>>);

impl<T> RawObject<T> {
    fn new<U>(ty: RawObject<TypeData>, data: U) -> Self {
        let inner = Box::new(Inner { rc: 1, ty, data });
        Self(NonNull::from(Box::leak(inner)).cast())
    }

    unsafe fn from_ptr(ptr: *mut Inner<T>) -> Self {
        Self(NonNull::new_unchecked(ptr))
    }

    fn unref(&self) {
        let inner = self.as_mut();
        inner.rc -= 1;
        if inner.rc == 0 {
            drop(unsafe { Box::from_raw(self.0.as_ptr()) });
        }
    }

    fn as_ref(&self) -> &Inner<T> {
        unsafe { self.0.as_ref() }
    }

    fn as_mut(&self) -> &mut Inner<T> {
        unsafe { &mut *self.0.as_ptr() }
    }

    unsafe fn cast<U>(&self) -> &Inner<U> {
        self.0.cast().as_ref()
    }

    unsafe fn cast_mut<U>(&mut self) -> &mut Inner<U> {
        self.0.cast().as_mut()
    }

    unsafe fn data<U>(&self) -> &U {
        &self.cast::<U>().data
    }

    unsafe fn data_mut<U>(&mut self) -> &mut U {
        &mut self.cast_mut::<U>().data
    }

    fn is_type(&self, ty: &RawObject<TypeData>) -> bool {
        self.as_ref().ty.0 == ty.0
    }

    fn type_data(&self) -> &TypeData {
        &self.as_ref().ty.as_ref().data
    }
}

impl<T> Clone for RawObject<T> {
    fn clone(&self) -> Self {
        let inner = self.as_mut();
        inner.rc += 1;
        Self(self.0)
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

    format: fn(&Object, &mut fmt::Formatter) -> fmt::Result,

    index: fn(&Object, &Object) -> Result<Object>,
    set_index: fn(&mut Object, &Object, Object) -> Result<()>,

    field: fn(&Object, &str) -> Result<Object>,
    set_field: fn(&mut Object, &str, Object) -> Result<()>,

    compare: fn(&Object, &Object) -> Result<Ordering>,

    contains: fn(&Object, &Object) -> Result<bool>,

    arithmetic: ArithmeticMethods,
}

impl Default for TypeData {
    fn default() -> Self {
        Self {
            name: String::new(),
            format: |_, f| f.write_str(""),
            index: |this, _| Err(unsupported(this, "index access")),
            set_index: |this, _, _| Err(unsupported(this, "index access")),
            field: |this, _| Err(unsupported(this, "field access")),
            set_field: |this, _, _| Err(unsupported(this, "field access")),
            compare: |this, _| Err(unsupported(this, "comparison")),
            contains: |this, _| Err(unsupported(this, "membership test")),
            arithmetic: ArithmeticMethods::default(),
        }
    }
}

struct ArithmeticMethods {
    not: fn(&Object) -> Result<Object>,
    or: fn(&Object, &Object) -> Result<Object>,
    xor: fn(&Object, &Object) -> Result<Object>,
    and: fn(&Object, &Object) -> Result<Object>,
    shl: fn(&Object, &Object) -> Result<Object>,
    shr: fn(&Object, &Object) -> Result<Object>,
    neg: fn(&Object) -> Result<Object>,
    add: fn(&Object, &Object) -> Result<Object>,
    sub: fn(&Object, &Object) -> Result<Object>,
    mul: fn(&Object, &Object) -> Result<Object>,
    div: fn(&Object, &Object) -> Result<Object>,
    rem: fn(&Object, &Object) -> Result<Object>,
}

impl Default for ArithmeticMethods {
    fn default() -> Self {
        Self {
            not: |this| Err(unsupported(this, "!")),
            or: |this, _| Err(unsupported(this, "|")),
            xor: |this, _| Err(unsupported(this, "^")),
            and: |this, _| Err(unsupported(this, "&")),
            shl: |this, _| Err(unsupported(this, "<<")),
            shr: |this, _| Err(unsupported(this, ">>")),
            neg: |this| Err(unsupported(this, "-")),
            add: |this, _| Err(unsupported(this, "+")),
            sub: |this, _| Err(unsupported(this, "-")),
            mul: |this, _| Err(unsupported(this, "*")),
            div: |this, _| Err(unsupported(this, "/")),
            rem: |this, _| Err(unsupported(this, "%")),
        }
    }
}

fn unsupported(this: &Object, op: &str) -> Error {
    Error::new(format!(
        "'{}' object doesn't support {} operation",
        this.type_name(),
        op
    ))
}

thread_local! {
    static INIT: Cell<bool> = Cell::new(false);

    static TYPE_TYPE: RawObject<TypeData> = unsafe {
        RawObject::from_ptr(TYPE_TYPE_DATA.with(|x| x.get()))
    };

    static TYPE_TYPE_DATA: UnsafeCell<Inner<TypeData>> = UnsafeCell::new(Inner {
        rc: 1,
        ty: RawObject(NonNull::dangling()),
        data: TypeData {
            name: "type".into(),
            format: |this, f| {
                let data = unsafe { this.0.data::<TypeData>() };
                write!(f, "{}", data.name)
            },
            ..Default::default()
        }
    });
}

pub(crate) fn init() {
    if !INIT.replace(true) {
        TYPE_TYPE_DATA.with(|x| unsafe { (*x.get()).ty = TYPE_TYPE.with(|t| t.clone()) });
    }
}
