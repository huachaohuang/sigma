use std::collections::HashMap;
use std::fs;

use serde_json::Value;

use crate::{Error, Object, Result};

pub(crate) fn module() -> Object {
    let mut module = HashMap::new();
    module.insert("load".into(), load.into());
    module.into()
}

fn load(_: &Object, args: &[Object]) -> Result<Object> {
    if args.len() != 1 {
        return Err(Error::new("expect 1 argument"));
    }
    let path = args[0]
        .as_str()
        .ok_or_else(|| Error::new("expect a path argument"))?;
    let text = fs::read_to_string(path)
        .map_err(|e| Error::new(format!("failed to read '{path}': {}", e.to_string())))?;
    let value = serde_json::from_str(&text)
        .map_err(|e| Error::new(format!("failed to parse json: {}", e.to_string())))?;
    value_to_object(value)
}

fn value_to_object(value: Value) -> Result<Object> {
    match value {
        Value::Null => Ok(().into()),
        Value::Bool(b) => Ok(b.into()),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into())
            } else if let Some(f) = n.as_f64() {
                Ok(f.into())
            } else {
                Err(Error::new("failed to convert json number to object"))
            }
        }
        Value::String(s) => Ok(s.into()),
        Value::Array(a) => a
            .into_iter()
            .map(value_to_object)
            .collect::<Result<Vec<_>>>()
            .map(|x| x.into()),
        Value::Object(o) => o
            .into_iter()
            .map(|(k, v)| value_to_object(v).map(|v| (k, v)))
            .collect::<Result<HashMap<_, _>>>()
            .map(|x| x.into()),
    }
}
