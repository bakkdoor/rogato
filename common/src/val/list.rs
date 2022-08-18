use std::{fmt::Display, hash::Hash};

use archery::{RcK, SharedPointer};

use super::ValueRef;
use crate::ast::ASTDepth;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct List {
    entries: rpds::List<ValueRef>,
}

type ListIter<'a> = std::iter::Map<
    rpds::list::IterPtr<'a, ValueRef, RcK>,
    for<'r> fn(&'r SharedPointer<ValueRef, RcK>) -> &'r ValueRef,
>;

impl List {
    pub fn iter(&self) -> ListIter<'_> {
        self.entries.iter()
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[ {} ]", self.entries))
    }
}
