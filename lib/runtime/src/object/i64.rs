use std::cell::UnsafeCell;

use super::*;

impl Object {
    pub(crate) fn as_i64(&self) -> Option<i64> {
        if TYPE.with(|t| self.0.is_type(t)) {
            Some(unsafe { *self.0.data::<i64>() })
        } else {
            None
        }
    }
}

impl From<i64> for Object {
    fn from(value: i64) -> Self {
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
            name: "i64".into(),
            format: |this, f| {
                let data = unsafe { this.0.data::<i64>() };
                write!(f, "{}", data)
            },
            ..Default::default()
        },
    });
}
