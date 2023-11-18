use std::cell::UnsafeCell;

use super::*;

impl From<bool> for Object {
    fn from(value: bool) -> Self {
        Object(RawObject::new(BOOL_TYPE.with(|t| t.clone()), value))
    }
}

thread_local! {
    static BOOL_TYPE: RawObject<TypeData> = unsafe {
        RawObject::from_ptr(BOOL_TYPE_DATA.with(|x| x.get()))
    };

    static BOOL_TYPE_DATA: UnsafeCell<Inner<TypeData>> = UnsafeCell::new(Inner {
        rc: 1,
        ty: RawObject::dangling(),
        data: TypeData {
            name: "bool".into(),
            format: |this, f| {
                let data = unsafe { this.0.cast_data::<bool>() };
                write!(f, "{}", data)
            },
        },
    });
}

pub(super) fn init() {
    BOOL_TYPE_DATA.with(|x| unsafe { (*x.get()).ty = BOOL_TYPE.with(|t| t.clone()) });
}
