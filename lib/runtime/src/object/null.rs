use std::cell::UnsafeCell;

use super::*;

impl From<()> for Object {
    fn from(_: ()) -> Self {
        NULL_TYPE.with(|x| x.clone()).into()
    }
}

thread_local! {
    static NULL_TYPE: RawObject<TypeData> = RawObject(unsafe {
        NonNull::new_unchecked(NULL_TYPE_DATA.with(|x| x.get()))
    });

    static NULL_TYPE_DATA: UnsafeCell<Inner<TypeData>> = UnsafeCell::new(Inner {
        rc: 1,
        ty: NULL_TYPE.with(|x| x.clone()),
        data: TypeData {
            name: "null".into(),
            fmt: |_, f| write!(f, "null"),
        }
    });
}
