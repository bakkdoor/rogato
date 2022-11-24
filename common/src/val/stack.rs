use std::{fmt::Display, hash::Hash};

use super::{Value, ValueRef};
use crate::ast::ASTDepth;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Stack {
    entries: rpds::Stack<ValueRef>,
}

type StackIter<'a> = rpds::stack::Iter<'a, ValueRef, archery::RcK>;

impl Stack {
    pub fn new() -> Self {
        Self {
            entries: rpds::Stack::new(),
        }
    }

    pub fn iter(&self) -> StackIter {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.size()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn peek(&self) -> Option<ValueRef> {
        self.entries.peek().map(ValueRef::clone)
    }

    pub fn push(&self, value: ValueRef) -> Self {
        Self {
            entries: self.entries.push(value),
        }
    }

    pub fn pop(&self) -> Option<Self> {
        self.entries.pop().map(|entries| Self { entries })
    }

    pub fn push_all(&self, other: &Self) -> Self {
        let mut joined = self.entries.clone();
        for val in other.iter() {
            joined = joined.push(ValueRef::clone(val))
        }
        Self { entries: joined }
    }
}

impl Default for Stack {
    fn default() -> Self {
        Stack::new()
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

impl From<Stack> for ValueRef {
    fn from(stack: Stack) -> Self {
        ValueRef::new(Value::Stack(stack))
    }
}
