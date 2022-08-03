use std::{collections::HashMap, fmt::Display, rc::Rc};

pub use serde_json::Number;

use crate::rogato::ast::{expression::TupleItems, ASTDepth};

pub fn list(vec: Vec<ValueRef>) -> ValueRef {
    Rc::new(Value::List(vec))
}

#[allow(dead_code)]
pub fn bool(b: bool) -> ValueRef {
    Rc::new(Value::Bool(b))
}

pub fn null() -> ValueRef {
    Rc::new(Value::Null)
}

pub fn int64(n: i64) -> ValueRef {
    Rc::new(Value::Int64(n))
}

pub fn object<S: ToString>(props: Vec<(S, ValueRef)>) -> ValueRef {
    let props: Vec<(String, ValueRef)> = props
        .iter()
        .map(|(prop, val)| (prop.to_string(), val.clone()))
        .collect();
    Rc::new(Value::Object(HashMap::from_iter(props)))
}

pub fn string<S: ToString>(s: S) -> ValueRef {
    Rc::new(Value::String(s.to_string()))
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Value {
    Null,
    String(String),
    Bool(bool),
    Int64(i64),
    List(Vec<ValueRef>),
    #[allow(dead_code)]
    Map(HashMap<String, ValueRef>),
    Object(HashMap<String, ValueRef>),
}

impl Value {
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}

pub type ValueRef = Rc<Value>;

impl From<serde_json::Value> for Value {
    fn from(json_val: serde_json::Value) -> Self {
        match json_val {
            serde_json::Value::Array(items) => Value::List(
                items
                    .iter()
                    .map(|item| Rc::new(Value::from(item.clone())))
                    .collect(),
            ),
            serde_json::Value::Bool(b) => Value::Bool(b),
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Number(n) => Value::Int64(n.as_i64().unwrap()),
            serde_json::Value::Object(props) => Value::Object(HashMap::from_iter(
                props
                    .iter()
                    .map(|(prop, val)| (prop.clone(), Rc::new(Value::from(val.clone()))))
                    .collect::<Vec<(String, ValueRef)>>(),
            )),
            serde_json::Value::String(s) => Value::String(s),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => f.write_str("null"),
            Value::String(s) => f.write_fmt(format_args!("\"{}\"", s)),
            Value::Bool(b) => f.write_fmt(format_args!("{}", b)),
            Value::Int64(i) => f.write_fmt(format_args!("{}", i)),
            Value::List(items) => {
                f.write_fmt(format_args!("[ {} ]", TupleItems::from(items.clone())))
            }
            Value::Map(items) => f.write_fmt(format_args!("{{ {:?} }}", items)),
            Value::Object(props) => f.write_fmt(format_args!("Object{{ {:?} }}", props)),
        }
    }
}

impl ASTDepth for Value {
    fn ast_depth(&self) -> usize {
        match self {
            Value::Null => 1,
            Value::String(_) => 1,
            Value::Bool(_) => 1,
            Value::Int64(_) => 1,
            Value::List(items) => 1 + items.len(),
            Value::Map(items) => 1 + items.len(),
            Value::Object(props) => 1 + props.len(),
        }
    }
}
