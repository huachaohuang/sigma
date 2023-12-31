use std::cell::{Cell, UnsafeCell};
use std::cmp::Ordering;
use std::fmt;
use std::ptr::NonNull;

use sigma_parser::ast::*;

mod bool;
mod f64;
mod func;
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

    pub(crate) fn call(&self, args: &[Object]) -> Result<Object> {
        (self.0.type_data().call)(self, args)
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

    pub(crate) fn cmpop(&self, op: CmpOp, other: &Object) -> Result<Object> {
        let value = match op {
            CmpOp::Eq => self.compare(other)? == Ordering::Equal,
            CmpOp::Ne => self.compare(other)? != Ordering::Equal,
            CmpOp::Lt => self.compare(other)? == Ordering::Less,
            CmpOp::Le => self.compare(other)? != Ordering::Greater,
            CmpOp::Gt => self.compare(other)? == Ordering::Greater,
            CmpOp::Ge => self.compare(other)? != Ordering::Less,
            CmpOp::In => other.contains(self)?,
            CmpOp::NotIn => !other.contains(self)?,
        };
        Ok(value.into())
    }

    fn compare(&self, other: &Object) -> Result<Ordering> {
        (self.0.type_data().compare)(self, other).ok_or_else(|| {
            Error::new(format!(
                "'{}' cannot be compared with '{}'",
                self.type_name(),
                other.type_name()
            ))
        })
    }

    pub(crate) fn iter(&self) -> Result<Iter> {
        (self.0.type_data().iter)(self)
    }

    pub(crate) fn iter_mut(&mut self) -> Result<IterMut> {
        (self.0.type_data().iter_mut)(self)
    }

    pub(crate) fn insert(&mut self, other: Object) -> Result<()> {
        (self.0.type_data().insert)(self, other)
    }

    pub(crate) fn replace(&mut self, other: Object) -> Result<()> {
        (self.0.type_data().replace)(self, other)
    }

    pub(crate) fn contains(&self, other: &Object) -> Result<bool> {
        (self.0.type_data().contains)(self, other)
    }
}

impl Drop for Object {
    fn drop(&mut self) {
        self.0.unref();
    }
}

impl Eq for Object {}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other)
            .map(|x| x == Ordering::Equal)
            .unwrap_or(false)
    }
}

impl PartialOrd for Object {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (self.0.type_data().compare)(self, other)
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
        let inner = unsafe { self.as_mut() };
        inner.rc -= 1;
        if inner.rc == 0 {
            drop(unsafe { Box::from_raw(self.0.as_ptr()) });
        }
    }

    unsafe fn as_ref(&self) -> &Inner<T> {
        unsafe { self.0.as_ref() }
    }

    unsafe fn as_mut(&self) -> &mut Inner<T> {
        &mut *self.0.as_ptr()
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
        unsafe { self.as_ref().ty.0 == ty.0 }
    }

    fn type_data(&self) -> &TypeData {
        unsafe { &self.as_ref().ty.as_ref().data }
    }
}

impl<T> Clone for RawObject<T> {
    fn clone(&self) -> Self {
        let inner = unsafe { self.as_mut() };
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

    call: fn(&Object, &[Object]) -> Result<Object>,

    index: fn(&Object, &Object) -> Result<Object>,
    set_index: fn(&mut Object, &Object, Object) -> Result<()>,

    field: fn(&Object, &str) -> Result<Object>,
    set_field: fn(&mut Object, &str, Object) -> Result<()>,

    compare: fn(&Object, &Object) -> Option<Ordering>,

    iter: for<'a> fn(&'a Object) -> Result<Iter<'a>>,
    iter_mut: fn(&mut Object) -> Result<IterMut>,

    insert: fn(&mut Object, Object) -> Result<()>,
    replace: fn(&mut Object, Object) -> Result<()>,
    contains: fn(&Object, &Object) -> Result<bool>,

    arithmetic: ArithmeticMethods,
}

impl Default for TypeData {
    fn default() -> Self {
        Self {
            name: String::new(),
            format: |this, f| write!(f, "<{}>", this.type_name()),
            call: |this, _| Err(unsupported(this, "is not callable")),
            index: |this, _| Err(unsupported_operation(this, "index access")),
            set_index: |this, _, _| Err(unsupported_operation(this, "index access")),
            field: |this, _| Err(unsupported_operation(this, "field access")),
            set_field: |this, _, _| Err(unsupported_operation(this, "field access")),
            compare: |_, _| None,
            iter: |this| Err(unsupported(this, "is not iterable")),
            iter_mut: |this| Err(unsupported(this, "is not iterable")),
            insert: |this, _| Err(unsupported_operation(this, "insert")),
            replace: |this, _| Err(unsupported_operation(this, "replace")),
            contains: |this, _| Err(unsupported_operation(this, "membership test")),
            arithmetic: ArithmeticMethods::default(),
        }
    }
}

pub(crate) type Iter<'a> = Box<dyn Iterator<Item = &'a Object> + 'a>;
pub(crate) type IterMut<'a> = Box<dyn Iterator<Item = &'a mut Object> + 'a>;

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
            not: |this| Err(unsupported_operation(this, "!")),
            or: |this, _| Err(unsupported_operation(this, "|")),
            xor: |this, _| Err(unsupported_operation(this, "^")),
            and: |this, _| Err(unsupported_operation(this, "&")),
            shl: |this, _| Err(unsupported_operation(this, "<<")),
            shr: |this, _| Err(unsupported_operation(this, ">>")),
            neg: |this| Err(unsupported_operation(this, "-")),
            add: |this, _| Err(unsupported_operation(this, "+")),
            sub: |this, _| Err(unsupported_operation(this, "-")),
            mul: |this, _| Err(unsupported_operation(this, "*")),
            div: |this, _| Err(unsupported_operation(this, "/")),
            rem: |this, _| Err(unsupported_operation(this, "%")),
        }
    }
}

fn unsupported(this: &Object, message: &str) -> Error {
    Error::new(format!("'{}' {}", this.type_name(), message))
}

fn unsupported_operation(this: &Object, op: &str) -> Error {
    Error::new(format!(
        "'{}' doesn't support {} operation",
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
