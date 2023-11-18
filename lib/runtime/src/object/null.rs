use std::cell::UnsafeCell;

use super::*;

impl From<()> for Object {
    fn from(_: ()) -> Self {
        Self(RawObject::new(NULL_TYPE.with(|t| t.clone()), ()))
    }
}

thread_local! {
    static NULL_TYPE: RawObject<TypeData> = unsafe {
        RawObject::from_ptr(NULL_TYPE_DATA.with(|x| x.get()))
    };

    static NULL_TYPE_DATA: UnsafeCell<Inner<TypeData>> = UnsafeCell::new(Inner {
        rc: 1,
        ty: RawObject::dangling(),
        data: TypeData {
            name: "null".into(),
            format: |_, f| write!(f, "null"),
        }
    });
}

pub(super) fn init() {
    NULL_TYPE_DATA.with(|x| unsafe { (*x.get()).ty = NULL_TYPE.with(|t| t.clone()) });
}
