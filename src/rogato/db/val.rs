use std::{collections::HashMap, rc::Rc};

pub use serde_json::{Number, Value};

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
    Value::Object(serde_json::Map::from_iter(props))
}

#[allow(dead_code)]
pub fn robject<S: ToString>(props: Vec<(S, Rc<RValue>)>) -> RValue {
    let props: Vec<(String, Rc<RValue>)> = props
        .iter()
        .map(|(prop, val)| (prop.to_string(), val.clone()))
        .collect();
    RValue::Object(HashMap::from_iter(props))
}

pub fn string<S: ToString>(s: S) -> Value {
    Value::String(s.to_string())
}

pub enum RValue {
    Null,
    String(String),
    Bool(bool),
    Int64(i64),
    List(Vec<Rc<RValue>>),
    #[allow(dead_code)]
    Map(HashMap<String, Rc<RValue>>),
    Object(HashMap<String, Rc<RValue>>),
}

impl From<serde_json::Value> for RValue {
    fn from(json_val: serde_json::Value) -> Self {
        match json_val {
            serde_json::Value::Array(items) => RValue::List(
                items
                    .iter()
                    .map(|item| Rc::new(RValue::from(item.clone())))
                    .collect(),
            ),
            serde_json::Value::Bool(b) => RValue::Bool(b),
            serde_json::Value::Null => RValue::Null,
            serde_json::Value::Number(n) => RValue::Int64(n.as_i64().unwrap()),
            serde_json::Value::Object(props) => RValue::Object(HashMap::from_iter(
                props
                    .iter()
                    .map(|(prop, val)| (prop.clone(), Rc::new(RValue::from(val.clone()))))
                    .collect::<Vec<(String, Rc<RValue>)>>(),
            )),
            serde_json::Value::String(s) => RValue::String(s),
        }
    }
}
