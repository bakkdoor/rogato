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
    pub fn new() -> Self {
        Self {
            entries: rpds::Queue::new(),
        }
    }

    pub fn iter(&self) -> QueueIter {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn enqueue(&self, value: ValueRef) -> Self {
        Self {
            entries: self.entries.enqueue(value),
        }
    }

    pub fn dequeue(&self) -> Option<Self> {
        self.entries.dequeue().map(|entries| Self { entries })
    }

    pub fn dequeue_or(&self, default: ValueRef) -> (ValueRef, Self) {
        let value = self.peek_or(default);
        let queue = self.dequeue().unwrap_or_default();
        (value, queue)
    }

    pub fn dequeue_or_else(&self, default: impl FnOnce() -> ValueRef) -> (ValueRef, Self) {
        let value = self.peek_or_else(default);
        let queue = self.dequeue().unwrap_or_default();
        (value, queue)
    }

    pub fn dequeue_or_insert(&self, default: ValueRef) -> (ValueRef, Self) {
        let value = self.peek_or(ValueRef::clone(&default));
        let queue = self.dequeue().unwrap_or_default();
        (value, queue.enqueue(default))
    }

    pub fn dequeue_or_insert_with(&self, default: impl FnOnce() -> ValueRef) -> (ValueRef, Self) {
        let value = self.peek_or_else(default);
        let queue = self.dequeue().unwrap_or_default();
        (ValueRef::clone(&value), queue.enqueue(value))
    }

    pub fn peek(&self) -> Option<ValueRef> {
        self.entries.peek().map(ValueRef::clone)
    }

    pub fn peek_or(&self, default: ValueRef) -> ValueRef {
        self.peek().unwrap_or(default)
    }

    pub fn peek_or_else(&self, default: impl FnOnce() -> ValueRef) -> ValueRef {
        self.peek().unwrap_or_else(default)
    }

    pub fn peek_or_insert(&self, default: ValueRef) -> (ValueRef, Self) {
        let value = self.peek_or(ValueRef::clone(&default));
        (value, self.enqueue(default))
    }

    pub fn peek_or_insert_with(&self, default: impl FnOnce() -> ValueRef) -> (ValueRef, Self) {
        let value = self.peek_or_else(default);
        (ValueRef::clone(&value), self.enqueue(value))
    }
}

impl Default for Queue {
    fn default() -> Self {
        Queue::new()
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

impl From<rpds::Queue<ValueRef>> for Queue {
    fn from(entries: rpds::Queue<ValueRef>) -> Self {
        Self { entries }
    }
}
