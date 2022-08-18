use std::{fmt::Display, hash::Hash};

use super::ValueRef;
use crate::ast::ASTDepth;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Vector {
    entries: rpds::Vector<ValueRef>,
}

impl FromIterator<ValueRef> for Vector {
    fn from_iter<T: IntoIterator<Item = ValueRef>>(iter: T) -> Self {
        Self {
            entries: rpds::Vector::from_iter(iter),
        }
    }
}

impl ASTDepth for Vector {
    fn ast_depth(&self) -> usize {
        1 + self.entries.len()
    }
}

impl Display for Vector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Vector{{ {} }}", self.entries))
    }
}
