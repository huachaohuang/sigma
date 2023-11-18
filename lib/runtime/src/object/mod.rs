use std::cell::{Cell, UnsafeCell};
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

impl Drop for Object {
    fn drop(&mut self) {
        self.0.unref();
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (unsafe { self.0.type_data() }.format)(self, f)
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
        println!("drop {:?} rc {}", self.0, inner.rc);
        inner.rc -= 1;
        if inner.rc == 0 {
            drop(unsafe { Box::from_raw(self.0.as_ptr()) });
        }
    }

    unsafe fn as_ref(&self) -> &Inner<T> {
        self.0.as_ref()
    }

    unsafe fn as_mut(&self) -> &mut Inner<T> {
        &mut *self.0.as_ptr()
    }

    unsafe fn cast_ref<U>(&self) -> &Inner<U> {
        self.0.cast().as_ref()
    }

    unsafe fn cast_data<U>(&self) -> &U {
        &self.cast_ref::<U>().data
    }

    unsafe fn type_data(&self) -> &TypeData {
        &self.as_ref().ty.as_ref().data
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

    format: FormatFn,
}

type FormatFn = fn(&Object, &mut fmt::Formatter) -> fmt::Result;

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
                let data = unsafe { this.0.cast_data::<TypeData>() };
                write!(f, "{}", data.name)
            }
        }
    });
}

pub(crate) fn init() {
    if !INIT.replace(true) {
        TYPE_TYPE_DATA.with(|x| unsafe { (*x.get()).ty = TYPE_TYPE.with(|t| t.clone()) });
    }
}
