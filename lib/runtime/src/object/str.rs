use std::cell::UnsafeCell;

use super::*;

impl From<&str> for Object {
    fn from(value: &str) -> Self {
        value.to_owned().into()
    }
}

impl From<String> for Object {
    fn from(value: String) -> Self {
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
            name: "str".into(),
            format: |this, f| {
                let data = unsafe { this.0.cast_data::<String>() };
                write!(f, "\"{}\"", data)
            },
        },
    });
}
