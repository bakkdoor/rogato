use std::{fmt::Display, hash::Hash};

use super::ValueRef;
use crate::ast::ASTDepth;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Stack {
    entries: rpds::Stack<ValueRef>,
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
