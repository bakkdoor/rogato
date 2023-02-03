use rust_decimal::prelude::*;
use std::cell::RefCell;
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

use crate::ast::lambda::{Lambda, LambdaClosureContext};
use crate::ast::{
    expression::{Expression, TupleItems},
    ASTDepth, Identifier, AST,
};
use crate::util::indent;

pub fn option(opt: Option<ValueRef>) -> ValueRef {
    ValueRef::new(Value::Option(opt))
}

pub fn none() -> ValueRef {
    ValueRef::new(Value::Option(None))
}

pub fn some(val: ValueRef) -> ValueRef {
    ValueRef::new(Value::Option(Some(val)))
}

pub fn string<S: ToString>(s: S) -> ValueRef {
    ValueRef::new(Value::String(s.to_string()))
}

pub fn symbol<ID: Into<Identifier>>(s: ID) -> ValueRef {
    let id: Identifier = s.into();
    if id.is_empty() {
        panic!("Blank identifier is not allowed as a symbol")
    }
    ValueRef::new(Value::Symbol(id))
}

pub fn bool(b: bool) -> ValueRef {
    ValueRef::new(Value::Bool(b))
}

pub fn number<Num>(n: Num) -> ValueRef
where
    Decimal: From<Num>,
{
    ValueRef::new(Value::Number(Decimal::from(n)))
}

pub fn decimal_str(s: &str) -> ValueRef {
    ValueRef::new(Value::Number(Decimal::from_str(s).unwrap()))
}

pub fn tuple<I: IntoIterator<Item = ValueRef>>(items: I) -> ValueRef {
    let vec: Vec<ValueRef> = items.into_iter().collect();
    ValueRef::new(Value::Tuple(vec.len(), vec))
}

pub fn list<I: IntoIterator<Item = ValueRef>>(items: I) -> ValueRef {
    ValueRef::new(Value::List(List::from_iter(items.into_iter())))
}

pub fn queue<I: IntoIterator<Item = ValueRef>>(items: I) -> ValueRef {
    ValueRef::new(Value::Queue(Queue::from_iter(items.into_iter())))
}

pub fn set<I: IntoIterator<Item = ValueRef>>(items: I) -> ValueRef {
    ValueRef::new(Value::Set(Set::from_iter(items.into_iter())))
}

pub fn stack<I: IntoIterator<Item = ValueRef>>(items: I) -> ValueRef {
    ValueRef::new(Value::Stack(Stack::from_iter(items.into_iter())))
}

pub fn vector<I: IntoIterator<Item = ValueRef>>(items: I) -> ValueRef {
    ValueRef::new(Value::Vector(Vector::from_iter(items.into_iter())))
}

pub fn map<I: IntoIterator<Item = (ValueRef, ValueRef)>>(items: I) -> ValueRef {
    ValueRef::new(Value::Map(Map::from_iter(items.into_iter())))
}

pub fn object<S: ToString, Props: IntoIterator<Item = (S, ValueRef)>>(props: Props) -> ValueRef {
    let props: Vec<(String, ValueRef)> = props
        .into_iter()
        .map(|(prop, val)| (prop.to_string(), val))
        .collect();
    ValueRef::new(Value::Object(Object::from_iter(props)))
}

pub fn lambda(ctx: Rc<RefCell<dyn LambdaClosureContext>>, l: Rc<Lambda>) -> ValueRef {
    ValueRef::new(Value::Lambda(ctx, l))
}

pub fn quoted(expr: Rc<Expression>) -> ValueRef {
    ValueRef::new(Value::Quoted(expr))
}
pub fn quoted_ast(ast: Rc<AST>) -> ValueRef {
    ValueRef::new(Value::QuotedAST(ast))
}

pub fn number_to_f32(num: &Decimal) -> Option<f32> {
    Decimal::to_f32(num)
}

pub fn number_to_f64(num: &Decimal) -> Option<f64> {
    Decimal::to_f64(num)
}

#[derive(Clone, Eq, std::fmt::Debug)]
pub enum Value {
    Option(Option<ValueRef>),
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
    Lambda(Rc<RefCell<dyn LambdaClosureContext>>, Rc<Lambda>),
    Quoted(Rc<Expression>),
    QuotedAST(Rc<AST>),
}

impl Value {
    pub fn is_none(&self) -> bool {
        matches!(self, Value::Option(None))
    }

    pub fn is_falsy(&self) -> bool {
        matches!(self, Value::Option(None) | Value::Bool(false))
    }

    pub fn is_truthy(&self) -> bool {
        !self.is_falsy()
    }
}

impl From<Option<ValueRef>> for Value {
    fn from(opt: Option<ValueRef>) -> Self {
        Value::Option(opt)
    }
}

pub type ValueRef = Rc<Value>;

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<serde_json::Value> for Value {
    fn from(json_val: serde_json::Value) -> Self {
        match json_val {
            serde_json::Value::Null => Value::Option(None),
            serde_json::Value::Array(items) => Value::List(List::from_iter(
                items
                    .iter()
                    .map(|item| ValueRef::new(Value::from(item.clone()))),
            )),
            serde_json::Value::Bool(b) => Value::Bool(b),
            serde_json::Value::Number(n) => Value::Number(Decimal::from(n.as_i64().unwrap())),
            serde_json::Value::Object(props) => Value::Object(Object::from_iter(
                props
                    .iter()
                    .map(|(prop, val)| (prop.clone(), ValueRef::new(Value::from(val.clone()))))
                    .collect::<Vec<(String, ValueRef)>>(),
            )),
            serde_json::Value::String(s) => Value::String(s),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Option(a), Value::Option(b)) => a.eq(b),
            (Value::String(a), Value::String(b)) => a.eq(b),
            (Value::Symbol(a), Value::Symbol(b)) => a.eq(b),
            (Value::Bool(a), Value::Bool(b)) => a.eq(b),
            (Value::Number(a), Value::Number(b)) => a.eq(b),
            (Value::Tuple(size_a, a), Value::Tuple(size_b, b)) => size_a.eq(size_b) && a.eq(b),
            (Value::List(a), Value::List(b)) => a.eq(b),
            (Value::Vector(a), Value::Vector(b)) => a.eq(b),
            (Value::Stack(a), Value::Stack(b)) => a.eq(b),
            (Value::Queue(a), Value::Queue(b)) => a.eq(b),
            (Value::Set(a), Value::Set(b)) => a.eq(b),
            (Value::Map(a), Value::Map(b)) => a.eq(b),
            (Value::Object(a), Value::Object(b)) => a.eq(b),
            (Value::Lambda(_, a), Value::Lambda(_, b)) => a.eq(b),
            (Value::Quoted(a), Value::Quoted(b)) => a.eq(b),
            (Value::QuotedAST(a), Value::QuotedAST(b)) => a.eq(b),
            _ => false,
        }
    }
}

impl Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
        match self {
            Value::Option(o) => {
                Hash::hash(&o, h);
            }
            Value::String(s) => {
                Hash::hash(&s, h);
            }
            Value::Symbol(s) => {
                Hash::hash(&s, h);
            }
            Value::Bool(b) => {
                Hash::hash(&b, h);
            }
            Value::Number(n) => {
                Hash::hash(&n, h);
            }
            Value::Tuple(size_a, items) => {
                Hash::hash(&size_a, h);
                Hash::hash(&items, h);
            }
            Value::List(l) => {
                Hash::hash(&l, h);
            }
            Value::Vector(v) => {
                Hash::hash(&v, h);
            }
            Value::Stack(s) => {
                Hash::hash(&s, h);
            }
            Value::Queue(q) => {
                Hash::hash(&q, h);
            }
            Value::Set(s) => {
                Hash::hash(&s, h);
            }
            Value::Map(m) => {
                Hash::hash(&m, h);
            }
            Value::Object(o) => {
                Hash::hash(&o, h);
            }
            Value::Lambda(ctx, l) => {
                Hash::hash(&ctx.as_ptr(), h);
                Hash::hash(&l, h);
            }
            Value::Quoted(q) => {
                Hash::hash(&q, h);
            }
            Value::QuotedAST(q) => {
                Hash::hash(&q, h);
            }
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Option(None) => f.write_str("None"),
            Value::Option(Some(val)) => {
                f.write_str("Some{ ")?;
                val.fmt(f)?;
                f.write_str(" }")
            }
            Value::String(s) => s.fmt(f),
            Value::Symbol(s) => {
                f.write_str("^")?;
                s.fmt(f)
            }
            Value::Bool(b) => b.fmt(f),
            Value::Number(d) => d.fmt(f),
            Value::Tuple(_size, items) => {
                let items: TupleItems<Value> = TupleItems::from(items.clone());
                if items.ast_depth() > 6 {
                    let items_str = format!("{items}");
                    if items_str.lines().count() == 1 {
                        f.write_fmt(format_args!("{{ {items} }}"))
                    } else {
                        f.write_fmt(format_args!("{{\n{}\n}}", indent(&items)))
                    }
                } else {
                    f.write_fmt(format_args!("{{ {items} }}"))
                }
            }
            Value::List(list) => list.fmt(f),
            Value::Vector(vector) => vector.fmt(f),
            Value::Stack(stack) => stack.fmt(f),
            Value::Queue(queue) => queue.fmt(f),
            Value::Set(set) => set.fmt(f),
            Value::Map(map) => map.fmt(f),
            Value::Object(object) => object.fmt(f),
            Value::Lambda(_, lambda) => lambda.fmt(f),
            Value::Quoted(expr) => {
                f.write_str("^")?;
                expr.fmt(f)
            }
            Value::QuotedAST(ast) => {
                f.write_str("^(")?;
                ast.fmt(f)?;
                f.write_str(")")
            }
        }
    }
}

impl ASTDepth for Value {
    fn ast_depth(&self) -> usize {
        match self {
            Value::Option(None) => 1,
            Value::Option(Some(val)) => 1 + val.ast_depth(),
            Value::String(_) => 1,
            Value::Symbol(_) => 1,
            Value::Bool(_) => 1,
            Value::Number(_) => 1,
            Value::Tuple(_, items) => 1 + items.iter().map(|i| i.ast_depth()).sum::<usize>(),
            Value::List(list) => list.ast_depth(),
            Value::Vector(vector) => vector.ast_depth(),
            Value::Stack(stack) => stack.ast_depth(),
            Value::Queue(queue) => queue.ast_depth(),
            Value::Set(set) => set.ast_depth(),
            Value::Map(items) => items.ast_depth(),
            Value::Object(object) => object.ast_depth(),
            Value::Lambda(_, lambda) => lambda.ast_depth(),
            Value::Quoted(expr) => 1 + expr.ast_depth(),
            Value::QuotedAST(ast) => 1 + ast.ast_depth(),
        }
    }
}
