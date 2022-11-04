use std::{fmt::Display, hash::Hash};

use super::{Value, ValueRef};
use crate::ast::ASTDepth;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Vector {
    entries: rpds::Vector<ValueRef>,
}

type VectorIter<'a> = rpds::vector::Iter<'a, ValueRef, archery::RcK>;

impl Vector {
    pub fn iter(&self) -> VectorIter {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
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
