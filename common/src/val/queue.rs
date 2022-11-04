use std::{
    fmt::Display,
    hash::{Hash, Hasher},
};

use super::{Value, ValueRef};
use crate::ast::ASTDepth;

#[derive(Clone, Eq, Debug)]
pub struct Queue {
    entries: rpds::Queue<ValueRef>,
}

type QueueIter<'a> = rpds::queue::Iter<'a, ValueRef, archery::RcK>;

impl Queue {
    pub fn iter(&self) -> QueueIter {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl FromIterator<ValueRef> for Queue {
    fn from_iter<T: IntoIterator<Item = ValueRef>>(iter: T) -> Self {
        Self {
            entries: rpds::Queue::from_iter(iter),
        }
    }
}

impl PartialEq for Queue {
    fn eq(&self, other: &Self) -> bool {
        self.entries.eq(&other.entries)
    }
}

impl Hash for Queue {
    fn hash<H: Hasher>(&self, h: &mut H) {
        let pairs: Vec<_> = self.entries.iter().collect();
        Hash::hash(&pairs, h);
    }
}

impl ASTDepth for Queue {
    fn ast_depth(&self) -> usize {
        1 + self.entries.len()
    }
}

impl Display for Queue {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;

        fmt.write_str("Queue[ ")?;

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

impl From<Queue> for ValueRef {
    fn from(queue: Queue) -> Self {
        ValueRef::new(Value::Queue(queue))
    }
}
