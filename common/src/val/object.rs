use crate::ast::ASTDepth;
use rpds::HashTrieMap;
use std::{
    fmt::Display,
    hash::{Hash, Hasher},
};

use super::{Value, ValueRef};

#[derive(Clone, Eq, Debug)]
pub struct Object {
    properties: HashTrieMap<String, ValueRef>,
}

impl FromIterator<(String, ValueRef)> for Object {
    fn from_iter<T: IntoIterator<Item = (String, ValueRef)>>(iter: T) -> Self {
        Object {
            properties: HashTrieMap::from_iter(iter),
        }
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        self.properties.eq(&other.properties)
    }
}

impl Hash for Object {
    fn hash<H: Hasher>(&self, h: &mut H) {
        let mut pairs: Vec<_> = self.properties.iter().collect();
        pairs.sort_by_key(|i| i.0);
        Hash::hash(&pairs, h);
    }
}

impl ASTDepth for Object {
    fn ast_depth(&self) -> usize {
        1 + self.properties.size()
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Object{{ {:?} }}", self.properties))
    }
}

impl From<Object> for ValueRef {
    fn from(queue: Object) -> Self {
        ValueRef::new(Value::Object(queue))
    }
}
