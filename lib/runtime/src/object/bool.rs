use std::cell::UnsafeCell;

use super::*;

impl From<bool> for Object {
    fn from(value: bool) -> Self {
        Self(RawObject::new(TYPE.with(|t| t.clone()), value))
    }
}

thread_local! {
    static TYPE: RawObject<TypeData> = unsafe {
        RawObject::from_ptr(TYPE_DATA.with(|x| x.get()))
    };

    static TYPE_DATA: UnsafeCell<Inner<TypeData>> = UnsafeCell::new(Inner {
        rc: 1,
        ty: super::TYPE_TYPE.with(|x| x.clone()),
        data: TypeData {
            name: "bool".into(),
            format: |this, f| {
                let data = unsafe { this.0.cast_data::<bool>() };
                write!(f, "{}", data)
            },
        },
    });
}
