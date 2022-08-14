use rust_decimal::prelude::*;
use std::{collections::HashMap, fmt::Display, rc::Rc};

use rust_decimal::Decimal;
pub use serde_json::Number;

use crate::ast::{expression::TupleItems, ASTDepth, Identifier};

pub fn tuple(vec: Vec<ValueRef>) -> ValueRef {
    Rc::new(Value::Tuple(vec.len(), vec))
}

pub fn list(vec: Vec<ValueRef>) -> ValueRef {
    Rc::new(Value::List(vec))
}

pub fn bool(b: bool) -> ValueRef {
    Rc::new(Value::Bool(b))
}

pub fn null() -> ValueRef {
    Rc::new(Value::Null)
}

pub fn decimal<Num>(n: Num) -> ValueRef
where
    Decimal: From<Num>,
{
    Rc::new(Value::Decimal(Decimal::from(n)))
}

pub fn decimal_str(s: &str) -> ValueRef {
    Rc::new(Value::Decimal(Decimal::from_str(s).unwrap()))
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

pub fn symbol(s: Identifier) -> ValueRef {
    Rc::new(Value::Symbol(s))
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Value {
    Null,
    String(String),
    Symbol(Identifier),
    Bool(bool),
    Decimal(Decimal),
    Tuple(usize, Vec<ValueRef>),
    List(Vec<ValueRef>),
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
            serde_json::Value::Number(n) => Value::Decimal(Decimal::from(n.as_i64().unwrap())),
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
            Value::Symbol(s) => f.write_fmt(format_args!("^{}", s)),
            Value::Bool(b) => f.write_fmt(format_args!("{}", b)),
            Value::Decimal(d) => f.write_fmt(format_args!("{}", d)),
            Value::Tuple(size, items) => f.write_fmt(format_args!(
                "{{{}}}{{ {} }}",
                size,
                TupleItems::from(items.clone())
            )),
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
            Value::Symbol(_) => 1,
            Value::Bool(_) => 1,
            Value::Decimal(_) => 1,
            Value::Tuple(size, _) => 1 + size,
            Value::List(items) => 1 + items.len(),
            Value::Map(items) => 1 + items.len(),
            Value::Object(props) => 1 + props.len(),
        }
    }
}
