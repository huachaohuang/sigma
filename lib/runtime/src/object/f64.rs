use std::cell::UnsafeCell;

use super::*;

impl Object {
    pub(crate) fn as_f64(&self) -> Option<f64> {
        if TYPE.with(|t| self.0.is_type(t)) {
            Some(unsafe { *self.0.data::<f64>() })
        } else {
            None
        }
    }
}

impl From<f64> for Object {
    fn from(value: f64) -> Self {
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
            name: "f64".into(),
            format,
            compare,
            arithmetic: ArithmeticMethods {
                neg: |this| unop(this, |x| -x),
                add: |this, other| binop(this, other, "+", |x, y| x + y),
                sub: |this, other| binop(this, other, "-", |x, y| x - y),
                mul: |this, other| binop(this, other, "*", |x, y| x * y),
                div: |this, other| binop(this, other, "/", |x, y| x / y),
                rem: |this, other| binop(this, other, "%", |x, y| x % y),
                ..Default::default()
            },
            ..Default::default()
        },
    });
}

fn format(this: &Object, f: &mut fmt::Formatter) -> fmt::Result {
    let data = unsafe { this.0.data::<f64>() };
    write!(f, "{}", data)
}

fn compare(this: &Object, other: &Object) -> Option<Ordering> {
    let data = unsafe { this.0.data::<f64>() };
    other.as_f64().map(|x| data.partial_cmp(&x).unwrap())
}

fn unop(this: &Object, f: fn(f64) -> f64) -> Result<Object> {
    let data = unsafe { this.0.data::<f64>() };
    Ok(f(*data).into())
}

fn binop(this: &Object, other: &Object, op: &str, f: fn(f64, f64) -> f64) -> Result<Object> {
    let data = unsafe { this.0.data::<f64>() };
    other.as_f64().map(|x| f(*data, x).into()).ok_or_else(|| {
        Error::new(format!(
            "invalid operands for operator '{}': '{}' and '{}'",
            op,
            this.type_name(),
            other.type_name()
        ))
    })
}
