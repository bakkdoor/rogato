use std::{fmt::Display, hash::Hash};

use super::ValueRef;
use crate::ast::ASTDepth;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Stack {
    entries: rpds::Stack<ValueRef>,
}

impl FromIterator<ValueRef> for Stack {
    fn from_iter<T: IntoIterator<Item = ValueRef>>(iter: T) -> Self {
        Self {
            entries: rpds::Stack::from_iter(iter),
        }
    }
}

impl ASTDepth for Stack {
    fn ast_depth(&self) -> usize {
        1 + self.entries.size()
    }
}

impl Display for Stack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Stack{{ {} }}", self.entries))
    }
}
