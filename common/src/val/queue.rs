use std::{fmt::Display, hash::Hash};

use super::ValueRef;
use crate::ast::ASTDepth;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Queue {
    entries: rpds::Queue<ValueRef>,
}

impl ASTDepth for Queue {
    fn ast_depth(&self) -> usize {
        1 + self.entries.len()
    }
}

impl Display for Queue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Queue{{ {} }}", self.entries))
    }
}
