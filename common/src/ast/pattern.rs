use rust_decimal::Decimal;

use super::{
    expression::{MapKVPair, TupleItems},
    ASTDepth, Identifier,
};
use std::{fmt::Display, rc::Rc};

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum Pattern {
    Any,
    EmptyList,
    ListCons(Rc<Pattern>, Rc<Pattern>),
    List(TupleItems<Pattern>),
    Tuple(usize, TupleItems<Pattern>),
    Map(TupleItems<MapKVPair<Pattern>>),
    MapCons(TupleItems<MapKVPair<Pattern>>, Rc<Pattern>),
    Var(Identifier),
    Bool(bool),
    Number(Decimal),
    String(String),
    Symbol(Identifier),
}

impl ASTDepth for Pattern {
    fn ast_depth(&self) -> usize {
        match self {
            Self::Any => 1,
            Self::EmptyList => 1,
            Self::ListCons(head, tail) => 1 + head.ast_depth() + tail.ast_depth(),
            Self::List(items) => 1 + items.ast_depth(),
            Self::Tuple(len, items) => 1 + len + items.ast_depth(),
            Self::Map(kv_pairs) => 1 + kv_pairs.ast_depth(),
            Self::MapCons(kv_pairs, rest) => 1 + kv_pairs.ast_depth() + rest.ast_depth(),
            Self::Var(_) => 1,
            Self::Bool(_) => 1,
            Self::Number(_) => 1,
            Self::String(_) => 1,
            Self::Symbol(_) => 1,
        }
    }
}

impl Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Any => f.write_str("_"),
            Self::EmptyList => f.write_str("[]"),
            Self::ListCons(head, tail) => {
                f.write_str("[ ")?;
                head.fmt(f)?;
                f.write_str(" :: ")?;
                tail.fmt(f)?;
                f.write_str(" ]")
            }
            Self::List(items) => {
                f.write_str("[ ")?;
                items.fmt(f)?;
                f.write_str(" ]")
            }
            Self::Tuple(_len, items) => {
                f.write_str("{ ")?;
                items.fmt(f)?;
                f.write_str(" }")
            }
            Self::Map(kv_pairs) => {
                f.write_str("{ ")?;
                kv_pairs.fmt(f)?;
                f.write_str(" }")
            }
            Self::MapCons(kv_pairs, rest) => {
                f.write_str("{ ")?;
                kv_pairs.fmt(f)?;
                f.write_str(" :: ")?;
                rest.fmt(f)?;
                f.write_str(" }")
            }
            Self::Var(id) => f.write_str(id),
            Self::Bool(b) => b.fmt(f),
            Self::Number(d) => d.fmt(f),
            Self::String(s) => s.fmt(f),
            Self::Symbol(s) => {
                f.write_str("^")?;
                s.fmt(f)
            }
        }
    }
}

impl<S: Into<Identifier>> From<S> for Pattern {
    fn from(s: S) -> Self {
        Pattern::Var(s.into())
    }
}
