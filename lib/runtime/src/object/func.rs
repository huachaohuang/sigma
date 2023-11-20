use std::cell::UnsafeCell;

use super::*;

type Func = Box<dyn Fn(&Object, &[Object]) -> Result<Object>>;

impl<T: Fn(&Object, &[Object]) -> Result<Object>> From<T> for Object {
    fn from(value: T) -> Self {
        Self(RawObject::new(TYPE.with(|t| t.clone()), Box::new(value)))
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
            name: "func".into(),
            call,
            ..Default::default()
        },
    });
}

fn call(this: &Object, args: &[Object]) -> Result<Object> {
    let func = unsafe { this.0.data::<Func>() };
    func(this, args)
}
