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

    pub fn contains(&self, key: &ValueRef) -> bool {
        self.entries.contains_key(key)
    }

    pub fn get(&self, key: &ValueRef) -> Option<ValueRef> {
        self.entries.get(key).map(ValueRef::clone)
    }

    pub fn get_or(&self, key: &ValueRef, default: ValueRef) -> ValueRef {
        self.get(key).unwrap_or(default)
    }

    pub fn get_or_else(&self, key: &ValueRef, default: impl FnOnce() -> ValueRef) -> ValueRef {
        self.get(key).unwrap_or_else(default)
    }

    pub fn get_or_insert(&self, key: ValueRef, default: ValueRef) -> (ValueRef, Self) {
        let value = self.get_or(&key, ValueRef::clone(&default));
        (value, self.insert(key, default))
    }

    pub fn get_or_insert_with(
        &self,
        key: ValueRef,
        default: impl FnOnce() -> ValueRef,
    ) -> (ValueRef, Self) {
        let value = self.get_or_else(&key, default);
        (ValueRef::clone(&value), self.insert(key, value))
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn insert(&self, key: ValueRef, value: ValueRef) -> Self {
        Self {
            entries: self.entries.insert(key, value),
        }
    }

    pub fn cons(&self, pairs: Vec<(ValueRef, ValueRef)>) -> Self {
        let mut map = self.clone();
        for (key, value) in pairs.iter() {
            map = map.insert(ValueRef::clone(key), ValueRef::clone(value))
        }
        map
    }

    pub fn remove(&self, key: &ValueRef) -> Self {
        Self {
            entries: self.entries.remove(key),
        }
    }

    pub fn merge(&self, other: &Map) -> Self {
        let mut entries = self.entries.clone();
        for (k, v) in other.entries.iter() {
            entries = entries.insert(ValueRef::clone(k), ValueRef::clone(v))
        }
        Self { entries }
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
            fmt.write_str(" : ")?;
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

impl From<HashTrieMap<ValueRef, ValueRef>> for Map {
    fn from(entries: HashTrieMap<ValueRef, ValueRef>) -> Self {
        Self { entries }
    }
}
