#[derive(Clone, Debug)]
pub struct Object;

impl From<&str> for Object {
    fn from(_: &str) -> Self {
        Object
    }
}

impl From<i64> for Object {
    fn from(_: i64) -> Self {
        Object
    }
}

impl From<f64> for Object {
    fn from(_: f64) -> Self {
        Object
    }
}

impl From<Vec<Object>> for Object {
    fn from(_: Vec<Object>) -> Self {
        Object
    }
}

impl From<Vec<(String, Object)>> for Object {
    fn from(_: Vec<(String, Object)>) -> Self {
        Object
    }
}
