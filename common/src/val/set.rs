use std::{
    fmt::Display,
    hash::{Hash, Hasher},
};

use super::{Value, ValueRef};
use crate::ast::ASTDepth;

#[derive(Clone, Eq, Debug)]
pub struct Set {
    entries: rpds::HashTrieSet<ValueRef>,
}

type SetIter<'a> = rpds::set::hash_trie_set::Iter<'a, ValueRef, archery::RcK>;

impl Set {
    pub fn new() -> Self {
        Set {
            entries: rpds::HashTrieSet::new(),
        }
    }

    pub fn iter(&self) -> SetIter {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.size()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn insert(&self, value: ValueRef) -> Self {
        Self {
            entries: self.entries.insert(value),
        }
    }

    pub fn remove(&self, value: &ValueRef) -> Self {
        Self {
            entries: self.entries.remove(value),
        }
    }

    pub fn contains(&self, value: &ValueRef) -> bool {
        self.entries.contains(value)
    }

    pub fn merge(&self, other: &Self) -> Self {
        let mut merged = self.entries.clone();
        for val in other.iter() {
            merged = merged.insert(ValueRef::clone(val))
        }
        Self { entries: merged }
    }

    pub fn is_disjoint(&self, other: &Self) -> bool {
        self.entries.is_disjoint(&other.entries)
    }

    pub fn is_subset(&self, other: &Self) -> bool {
        self.entries.is_subset(&other.entries)
    }

    pub fn is_superset(&self, other: &Self) -> bool {
        self.entries.is_superset(&other.entries)
    }
}

impl Default for Set {
    fn default() -> Self {
        Set::new()
    }
}

impl FromIterator<ValueRef> for Set {
    fn from_iter<T: IntoIterator<Item = ValueRef>>(iter: T) -> Self {
        Self {
            entries: rpds::HashTrieSet::from_iter(iter),
        }
    }
}

impl PartialEq for Set {
    fn eq(&self, other: &Self) -> bool {
        self.entries.eq(&other.entries)
    }
}

impl Hash for Set {
    fn hash<H: Hasher>(&self, h: &mut H) {
        let mut pairs: Vec<_> = self.entries.iter().collect();
        // TODO: improve this, probably need to implement Ord for Value
        pairs.sort_by_key(|val| val.to_string());
        Hash::hash(&pairs, h);
    }
}

impl ASTDepth for Set {
    fn ast_depth(&self) -> usize {
        1 + self.entries.size()
    }
}

impl Display for Set {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;

        fmt.write_str("Set[ ")?;

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

impl From<Set> for ValueRef {
    fn from(set: Set) -> Self {
        ValueRef::new(Value::Set(set))
    }
}

impl From<rpds::HashTrieSet<ValueRef>> for Set {
    fn from(entries: rpds::HashTrieSet<ValueRef>) -> Self {
        Self { entries }
    }
}
