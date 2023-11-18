use std::cell::UnsafeCell;
use std::collections::HashMap;

use super::*;

type Hash = HashMap<String, Object>;

impl From<Hash> for Object {
    fn from(value: Hash) -> Self {
        Self(RawObject::new(TYPE.with(|t| t.clone()), value))
    }
}

impl From<Vec<(String, Object)>> for Object {
    fn from(value: Vec<(String, Object)>) -> Self {
        Hash::from_iter(value).into()
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
            name: "hash".into(),
            format,
            index,
            set_index,
            field,
            set_field,
            contains,
            ..Default::default()
        },
    });
}

fn format(this: &Object, f: &mut fmt::Formatter) -> fmt::Result {
    let hash = unsafe { this.0.data::<Hash>() };
    f.write_str("{")?;
    for (i, (k, v)) in hash.iter().enumerate() {
        if i > 0 {
            f.write_str(", ")?;
        }
        write!(f, "{}: {}", k, v)?;
    }
    f.write_str("}")
}

fn index(this: &Object, index: &Object) -> Result<Object> {
    hash_index(index).and_then(|x| field(this, x))
}

fn set_index(this: &mut Object, index: &Object, value: Object) -> Result<()> {
    hash_index(index).and_then(|x| set_field(this, x, value))
}

fn hash_index(index: &Object) -> Result<&str> {
    index.as_str().ok_or_else(|| {
        Error::new(format!(
            "hash index must be 'str', not '{}'",
            index.type_name()
        ))
    })
}

fn field(this: &Object, field: &str) -> Result<Object> {
    let hash = unsafe { this.0.data::<Hash>() };
    hash.get(field)
        .cloned()
        .ok_or_else(|| Error::new(format!("field '{field}' is not found")))
}

fn set_field(this: &mut Object, field: &str, value: Object) -> Result<()> {
    let hash = unsafe { this.0.data_mut::<Hash>() };
    hash.insert(field.into(), value);
    Ok(())
}

fn contains(this: &Object, other: &Object) -> Result<bool> {
    let hash = unsafe { this.0.data::<Hash>() };
    Ok(other
        .as_str()
        .map(|x| hash.contains_key(x))
        .unwrap_or(false))
}
