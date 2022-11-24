use std::{fmt::Display, hash::Hash};

use super::{Value, ValueRef};
use crate::ast::ASTDepth;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Vector {
    entries: rpds::Vector<ValueRef>,
}

type VectorIter<'a> = rpds::vector::Iter<'a, ValueRef, archery::RcK>;

impl Vector {
    pub fn new() -> Self {
        Vector {
            entries: rpds::Vector::new(),
        }
    }

    pub fn iter(&self) -> VectorIter {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn push_back(&self, value: ValueRef) -> Self {
        Self {
            entries: self.entries.push_back(value),
        }
    }

    pub fn drop_last(&self) -> Option<Self> {
        self.entries.drop_last().map(|entries| Self { entries })
    }

    pub fn append_all(&self, other: &Self) -> Self {
        let mut appended = other.entries.clone();
        for val in self.iter() {
            appended = appended.push_back(ValueRef::clone(val))
        }
        Self { entries: appended }
    }

    pub fn first(&self) -> Option<ValueRef> {
        self.entries.first().map(ValueRef::clone)
    }

    pub fn last(&self) -> Option<ValueRef> {
        self.entries.last().map(ValueRef::clone)
    }

    pub fn get(&self, idx: usize) -> Option<ValueRef> {
        self.entries.get(idx).map(ValueRef::clone)
    }

    pub fn set(&self, idx: usize, value: ValueRef) -> Option<Self> {
        self.entries.set(idx, value).map(|entries| Self { entries })
    }
}

impl Default for Vector {
    fn default() -> Self {
        Vector::new()
    }
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
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;

        fmt.write_str("Vector[ ")?;

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

impl From<Vector> for ValueRef {
    fn from(vector: Vector) -> Self {
        ValueRef::new(Value::Vector(vector))
    }
}

impl From<rpds::Vector<ValueRef>> for Vector {
    fn from(entries: rpds::Vector<ValueRef>) -> Self {
        Self { entries }
    }
}
