use std::cell::UnsafeCell;

use super::*;

thread_local! {
    static TYPE_TYPE: RawObject<TypeData> = unsafe {
        RawObject::from_ptr(TYPE_TYPE_DATA.with(|x| x.get()))
    };

    static TYPE_TYPE_DATA: UnsafeCell<Inner<TypeData>> = UnsafeCell::new(Inner {
        rc: 1,
        ty: RawObject::dangling(),
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
    TYPE_TYPE_DATA.with(|x| unsafe { (*x.get()).ty = TYPE_TYPE.with(|t| t.clone()) });
}
