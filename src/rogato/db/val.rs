pub use serde_json::{Map, Number, Value};

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
pub fn number<N>(n: N) -> Value
where
    Number: From<N>,
{
    Value::Number(Number::from(n))
}

#[allow(dead_code)]
pub fn object<S: ToString>(props: Vec<(S, Value)>) -> Value {
    let props: Vec<(String, Value)> = props
        .iter()
        .map(|(prop, val)| (prop.to_string(), val.clone()))
        .collect();
    Value::Object(Map::from_iter(props))
}

#[allow(dead_code)]
pub fn string<S: ToString>(s: S) -> Value {
    Value::String(s.to_string())
}
