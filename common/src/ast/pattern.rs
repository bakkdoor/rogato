use super::{expression::TupleItems, ASTDepth, Identifier};
use std::{fmt::Display, rc::Rc};

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum Pattern {
    AnyPattern,
    EmptyList,
    ListCons(Rc<Pattern>, Rc<Pattern>),
    ListLit(TupleItems<Pattern>),
    Var(Identifier),
}

impl ASTDepth for Pattern {
    fn ast_depth(&self) -> usize {
        match self {
            Self::AnyPattern => 1,
            Self::EmptyList => 1,
            Self::ListCons(head, tail) => 1 + head.ast_depth() + tail.ast_depth(),
            Self::ListLit(items) => 1 + items.ast_depth(),
            Self::Var(_) => 1,
        }
    }
}

impl Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AnyPattern => f.write_str("_"),
            Self::EmptyList => f.write_str("[]"),
            Self::ListCons(head, tail) => {
                f.write_str("[ ")?;
                head.fmt(f)?;
                f.write_str(" :: ")?;
                tail.fmt(f)?;
                f.write_str("[ ")
            }
            Self::ListLit(items) => {
                f.write_str("[ ")?;
                items.fmt(f)?;
                f.write_str("[ ")
            }
            Self::Var(id) => f.write_str(id),
        }
    }
}

impl<S: Into<Identifier>> From<S> for Pattern {
    fn from(s: S) -> Self {
        Pattern::Var(s.into())
    }
}
