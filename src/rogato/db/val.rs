pub use serde_json::{Map, Number, Value};

pub fn array(vec: Vec<Value>) -> Value {
    Value::Array(vec)
}

pub fn bool(b: bool) -> Value {
    Value::Bool(b)
}

pub fn null() -> Value {
    Value::Null
}

pub fn number<N>(n: N) -> Value
where
    Number: From<N>,
{
    Value::Number(Number::from(n))
}

pub fn object<S: ToString>(props: Vec<(S, Value)>) -> Value {
    let props: Vec<(String, Value)> = props
        .iter()
        .map(|(prop, val)| (prop.to_string(), val.clone()))
        .collect();
    Value::Object(Map::from_iter(props))
}

pub fn string<S: ToString>(s: S) -> Value {
    Value::String(s.to_string())
}
