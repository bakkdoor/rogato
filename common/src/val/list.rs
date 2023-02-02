use std::{fmt::Display, hash::Hash, rc::Rc};

use super::{Value, ValueRef};
use crate::{
    ast::{expression::TupleItems, ASTDepth},
    util::indent,
};

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct List {
    entries: rpds::List<ValueRef>,
}

type ListIter<'a> = rpds::list::Iter<'a, ValueRef, archery::RcK>;

impl List {
    pub fn new() -> Self {
        Self {
            entries: rpds::List::new(),
        }
    }

    pub fn iter(&self) -> ListIter<'_> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn head(&self) -> Option<ValueRef> {
        self.entries.first().map(Rc::clone)
    }

    pub fn tail(&self) -> Self {
        if let Some(entries) = self.entries.drop_first() {
            return List { entries };
        }
        List::from_iter([])
    }

    pub fn cons(&self, value: ValueRef) -> Self {
        List {
            entries: self.entries.push_front(value),
        }
    }

    pub fn reverse(&self) -> Self {
        List {
            entries: self.entries.reverse(),
        }
    }

    pub fn join(&self, other: &Self) -> Self {
        let mut joined = other.entries.clone();
        for e in self.entries.reverse().iter() {
            joined = joined.push_front(ValueRef::clone(e))
        }
        Self { entries: joined }
    }

    pub fn chunks(&self, chunk_size: usize) -> Vec<Self> {
        let mut chunks = Vec::with_capacity(self.len() / chunk_size);
        let entries: Vec<ValueRef> = self.entries.iter().cloned().collect();

        for chunk in entries.chunks(chunk_size) {
            chunks.push(List::from_iter(chunk.iter().cloned()));
        }

        chunks
    }

    pub fn contains(&self, item: &ValueRef) -> bool {
        self.entries.iter().any(|i| i.eq(item))
    }
}

impl Default for List {
    fn default() -> Self {
        List::new()
    }
}

impl From<List> for ValueRef {
    fn from(list: List) -> Self {
        ValueRef::new(Value::List(list))
    }
}

impl From<rpds::List<ValueRef>> for List {
    fn from(entries: rpds::List<ValueRef>) -> Self {
        Self { entries }
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
        1 + self.entries.iter().map(|i| i.ast_depth()).sum::<usize>()
    }
}

impl Display for List {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let items: TupleItems<Value> = TupleItems::from_iter(self.iter().map(Rc::clone));

        if items.ast_depth() > 6 {
            let items_str = format!("{items}");
            if items_str.lines().count() == 1 {
                fmt.write_fmt(format_args!("[ {items} ]"))
            } else {
                fmt.write_fmt(format_args!("[\n{}\n]", indent(&items)))
            }
        } else {
            fmt.write_fmt(format_args!("[ {items} ]"))
        }
    }
}
