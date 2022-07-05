use serde_json::{Map, Number, Value};

#[allow(dead_code)]
pub fn array(vec: Vec<Value>) -> Value {
    Value::Array(vec)
}

#[allow(dead_code)]
pub fn bool(b: bool) -> Value {
    Value::Bool(b)
}

#[allow(dead_code)]
pub fn null() -> Value {
    Value::Null
}

#[allow(dead_code)]
pub fn number(n: Number) -> Value {
    Value::Number(n)
}

#[allow(dead_code)]
pub fn object(props: Map<String, Value>) -> Value {
    Value::Object(props)
}

#[allow(dead_code)]
pub fn string<S: ToString>(s: S) -> Value {
    Value::String(s.to_string())
}
