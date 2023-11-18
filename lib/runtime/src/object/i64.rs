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
            format,
            compare,
            arithmetic: ArithmeticMethods {
                not: |this| unop(this, |x| !x),
                or: |this, other| binop(this, other, "|", |x, y| x | y),
                xor: |this, other| binop(this, other, "^", |x, y| x ^ y),
                and: |this, other| binop(this, other, "&", |x, y| x & y),
                shl: |this, other| binop(this, other, "<<", |x, y| x << y),
                shr: |this, other| binop(this, other, ">>", |x, y| x >> y),
                neg: |this| unop(this, |x| -x),
                add: |this, other| binop(this, other, "+", |x, y| x + y),
                sub: |this, other| binop(this, other, "-", |x, y| x - y),
                mul: |this, other| binop(this, other, "*", |x, y| x * y),
                div: |this, other| binop(this, other, "/", |x, y| x / y),
                rem: |this, other| binop(this, other, "%", |x, y| x % y),
            },
            ..Default::default()
        },
    });
}

fn format(this: &Object, f: &mut fmt::Formatter) -> fmt::Result {
    let data = unsafe { this.0.data::<i64>() };
    write!(f, "{}", data)
}

fn compare(this: &Object, other: &Object) -> Option<Ordering> {
    let data = unsafe { this.0.data::<i64>() };
    other.as_i64().map(|x| data.cmp(&x))
}

fn unop(this: &Object, f: fn(i64) -> i64) -> Result<Object> {
    let data = unsafe { this.0.data::<i64>() };
    Ok(f(*data).into())
}

fn binop(this: &Object, other: &Object, op: &str, f: fn(i64, i64) -> i64) -> Result<Object> {
    let data = unsafe { this.0.data::<i64>() };
    other.as_i64().map(|x| f(*data, x).into()).ok_or_else(|| {
        Error::new(format!(
            "invalid operands for operator '{}': '{}' and '{}'",
            op,
            this.type_name(),
            other.type_name()
        ))
    })
}
