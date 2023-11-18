use std::cell::UnsafeCell;

use super::*;

type List = Vec<Object>;

impl From<List> for Object {
    fn from(value: List) -> Self {
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
            name: "list".into(),
            format,
            index,
            set_index,
            ..Default::default()
        },
    });
}

fn format(this: &Object, f: &mut fmt::Formatter) -> fmt::Result {
    let data = unsafe { this.0.data::<List>() };
    f.write_str("[")?;
    for (i, item) in data.iter().enumerate() {
        if i > 0 {
            f.write_str(", ")?;
        }
        write!(f, "{}", item)?;
    }
    f.write_str("]")
}

fn index(this: &Object, index: &Object) -> Result<Object> {
    let list = unsafe { this.0.data::<List>() };
    let i = len_index(list.len(), index)?;
    Ok(unsafe { list.get_unchecked(i).clone() })
}

fn set_index(this: &mut Object, index: &Object, value: Object) -> Result<()> {
    let list = unsafe { this.0.data_mut::<List>() };
    let i = len_index(list.len(), index)?;
    Ok(unsafe { *list.get_unchecked_mut(i) = value })
}

fn len_index<'a>(len: usize, index: &Object) -> Result<usize> {
    match index.as_i64() {
        Some(i) => {
            let i = if i >= 0 {
                i as usize
            } else {
                len - (-i as usize)
            };
            if i < len {
                Ok(i)
            } else {
                Err(Error::new(format!("index '{i}' out of bounds")))
            }
        }
        None => Err(Error::new("index must be an integer")),
    }
}
