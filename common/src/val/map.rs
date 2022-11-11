use rpds::HashTrieMap;
use std::{
    fmt::Display,
    hash::{Hash, Hasher},
};

use crate::ast::ASTDepth;

use super::{Value, ValueRef};

#[derive(Clone, Eq, Debug)]
pub struct Map {
    entries: HashTrieMap<ValueRef, ValueRef>,
}

type MapIter<'a> = rpds::map::hash_trie_map::Iter<'a, ValueRef, ValueRef, archery::RcK>;

impl Map {
    pub fn new() -> Self {
        Self {
            entries: HashTrieMap::new(),
        }
    }

    pub fn iter(&self) -> MapIter {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.size()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for Map {
    fn default() -> Self {
        Map::new()
    }
}

impl FromIterator<(ValueRef, ValueRef)> for Map {
    fn from_iter<T: IntoIterator<Item = (ValueRef, ValueRef)>>(iter: T) -> Self {
        Self {
            entries: rpds::HashTrieMap::from_iter(iter),
        }
    }
}

impl PartialEq for Map {
    fn eq(&self, other: &Self) -> bool {
        self.entries.eq(&other.entries)
    }
}

impl Hash for Map {
    fn hash<H: Hasher>(&self, h: &mut H) {
        let mut pairs: Vec<_> = self.entries.iter().collect();
        // TODO: improve this, probably need to implement Ord for Value
        pairs.sort_by_key(|i| i.0.to_string());
        Hash::hash(&pairs, h);
    }
}

impl ASTDepth for Map {
    fn ast_depth(&self) -> usize {
        self.entries.size()
    }
}

impl Display for Map {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;

        fmt.write_str("{ ")?;

        for (k, v) in self.iter() {
            if !first {
                fmt.write_str(", ")?;
            }
            k.fmt(fmt)?;
            fmt.write_str(" => ")?;
            v.fmt(fmt)?;
            first = false;
        }

        fmt.write_str(" }")
    }
}

impl From<Map> for ValueRef {
    fn from(map: Map) -> Self {
        ValueRef::new(Value::Map(map))
    }
}
