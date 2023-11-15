use std::cell::UnsafeCell;

use super::*;

impl From<bool> for Object {
    fn from(value: bool) -> Self {
        RawObject::new(BOOL_TYPE.with(Clone::clone), value).into()
    }
}

thread_local! {
    static BOOL_TYPE: RawObject<TypeData> = RawObject(unsafe {
        NonNull::new_unchecked(BOOL_TYPE_DATA.with(|x| x.get()))
    });

    static BOOL_TYPE_DATA: UnsafeCell<Inner<TypeData>> = UnsafeCell::new(Inner {
        rc: 1,
        ty: TYPE_TYPE.with(|x| x.clone()),
        data: TypeData {
            name: "bool".into(),
            fmt: |this, f| {
                let data = unsafe { this.0.data::<bool>() };
                write!(f, "{}", data)
            },
        },
    });
}
