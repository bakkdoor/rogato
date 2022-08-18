use rust_decimal::prelude::*;
use std::hash::Hash;
use std::{fmt::Display, rc::Rc};

use rust_decimal::Decimal;
pub use serde_json::Number;

pub mod list;
pub mod map;
pub mod object;
pub mod queue;
pub mod set;
pub mod stack;
pub mod vector;

pub use list::List;
pub use map::Map;
pub use object::Object;
pub use queue::Queue;
pub use set::Set;
pub use stack::Stack;
pub use vector::Vector;

use crate::ast::{
    expression::{Expression, Lambda, TupleItems},
    ASTDepth, Identifier, AST,
};

pub fn null() -> ValueRef {
    Rc::new(Value::Null)
}

pub fn string<S: ToString>(s: S) -> ValueRef {
    Rc::new(Value::String(s.to_string()))
}

pub fn symbol(s: Identifier) -> ValueRef {
    Rc::new(Value::Symbol(s))
}

pub fn bool(b: bool) -> ValueRef {
    Rc::new(Value::Bool(b))
}

pub fn number<Num>(n: Num) -> ValueRef
where
    Decimal: From<Num>,
{
    Rc::new(Value::Number(Decimal::from(n)))
}

pub fn decimal_str(s: &str) -> ValueRef {
    Rc::new(Value::Number(Decimal::from_str(s).unwrap()))
}

pub fn tuple(vec: Vec<ValueRef>) -> ValueRef {
    Rc::new(Value::Tuple(vec.len(), vec))
}

pub fn list<I: IntoIterator<Item = ValueRef>>(items: I) -> ValueRef {
    Rc::new(Value::List(List::from_iter(items.into_iter())))
}

pub fn queue<I: IntoIterator<Item = ValueRef>>(items: I) -> ValueRef {
    Rc::new(Value::Queue(Queue::from_iter(items.into_iter())))
}

pub fn set<I: IntoIterator<Item = ValueRef>>(items: I) -> ValueRef {
    Rc::new(Value::Set(Set::from_iter(items.into_iter())))
}

pub fn stack<I: IntoIterator<Item = ValueRef>>(items: I) -> ValueRef {
    Rc::new(Value::Stack(Stack::from_iter(items.into_iter())))
}

pub fn vector<I: IntoIterator<Item = ValueRef>>(items: I) -> ValueRef {
    Rc::new(Value::Vector(Vector::from_iter(items.into_iter())))
}

pub fn map<I: IntoIterator<Item = (ValueRef, ValueRef)>>(items: I) -> ValueRef {
    Rc::new(Value::Map(Map::from_iter(items.into_iter())))
}

pub fn object<S: ToString>(props: Vec<(S, ValueRef)>) -> ValueRef {
    let props: Vec<(String, ValueRef)> = props
        .iter()
        .map(|(prop, val)| (prop.to_string(), val.clone()))
        .collect();
    Rc::new(Value::Object(Object::from_iter(props)))
}

pub fn lambda(l: Rc<Lambda>) -> ValueRef {
    Rc::new(Value::Lambda(l))
}

pub fn quoted(expr: Rc<Expression>) -> ValueRef {
    Rc::new(Value::Quoted(expr))
}
pub fn quoted_ast(ast: Rc<AST>) -> ValueRef {
    Rc::new(Value::QuotedAST(ast))
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum Value {
    Null,
    String(String),
    Symbol(Identifier),
    Bool(bool),
    Number(Decimal),
    Tuple(usize, Vec<ValueRef>),
    List(List),
    Vector(Vector),
    Stack(Stack),
    Queue(Queue),
    Set(Set),
    Map(Map),
    Object(Object),
    Lambda(Rc<Lambda>),
    Quoted(Rc<Expression>),
    QuotedAST(Rc<AST>),
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
            serde_json::Value::Array(items) => Value::List(List::from_iter(
                items.iter().map(|item| Rc::new(Value::from(item.clone()))),
            )),
            serde_json::Value::Bool(b) => Value::Bool(b),
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Number(n) => Value::Number(Decimal::from(n.as_i64().unwrap())),
            serde_json::Value::Object(props) => Value::Object(Object::from_iter(
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
            Value::Number(d) => f.write_fmt(format_args!("{}", d)),
            Value::Tuple(size, items) => f.write_fmt(format_args!(
                "{{{}}}{{ {} }}",
                size,
                TupleItems::from(items.clone())
            )),
            Value::List(list) => f.write_fmt(format_args!(
                "[ {} ]",
                TupleItems::from_iter(list.iter().map(Rc::clone))
            )),
            Value::Vector(vector) => f.write_fmt(format_args!("{}", vector)),
            Value::Stack(stack) => f.write_fmt(format_args!("{}", stack)),
            Value::Queue(queue) => f.write_fmt(format_args!("{}", queue)),
            Value::Set(set) => f.write_fmt(format_args!("{}", set)),
            Value::Map(items) => f.write_fmt(format_args!("{{ {:?} }}", items)),
            Value::Object(props) => f.write_fmt(format_args!("Object{{ {:?} }}", props)),
            Value::Lambda(lambda) => f.write_fmt(format_args!("{}", lambda)),
            Value::Quoted(expr) => f.write_fmt(format_args!("^{}", expr)),
            Value::QuotedAST(ast) => f.write_fmt(format_args!("^({})", ast)),
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
            Value::Number(_) => 1,
            Value::Tuple(size, _) => 1 + size,
            Value::List(list) => list.ast_depth(),
            Value::Vector(vector) => vector.ast_depth(),
            Value::Stack(stack) => stack.ast_depth(),
            Value::Queue(queue) => queue.ast_depth(),
            Value::Set(set) => set.ast_depth(),
            Value::Map(items) => items.ast_depth(),
            Value::Object(object) => object.ast_depth(),
            Value::Lambda(lambda) => lambda.ast_depth(),
            Value::Quoted(expr) => 1 + expr.ast_depth(),
            Value::QuotedAST(ast) => 1 + ast.ast_depth(),
        }
    }
}
