use std::{fmt::Display, hash::Hash};

use super::ValueRef;
use crate::ast::ASTDepth;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct List {
    entries: rpds::List<ValueRef>,
}

type ListIter<'a> = rpds::list::Iter<'a, ValueRef, archery::RcK>;

impl List {
    pub fn iter(&self) -> ListIter<'_> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

impl FromIterator<ValueRef> for List {
    fn from_iter<T: IntoIterator<Item = ValueRef>>(iter: T) -> Self {
        Self {
            entries: rpds::List::from_iter(iter),
        }
    }
}

impl ASTDepth for List {
    fn ast_depth(&self) -> usize {
        1 + self.entries.len()
    }
}

impl Display for List {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;

        fmt.write_str("[ ")?;

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
