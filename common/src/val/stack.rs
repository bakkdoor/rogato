use std::{fmt::Display, hash::Hash};

use super::ValueRef;
use crate::ast::ASTDepth;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Stack {
    entries: rpds::Stack<ValueRef>,
}

type StackIter<'a> = rpds::stack::Iter<'a, ValueRef, archery::RcK>;

impl Stack {
    pub fn iter(&self) -> StackIter {
        self.entries.iter()
    }
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
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;

        fmt.write_str("Stack[ ")?;

        for v in self.iter() {
            if !first {
                fmt.write_str(", ")?;
            }
            v.fmt(fmt)?;
            first = false;
        }

        fmt.write_str(" ]")
    }
}
