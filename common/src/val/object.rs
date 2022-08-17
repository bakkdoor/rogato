use crate::ast::ASTDepth;
use rpds::HashTrieMap;
use std::hash::{Hash, Hasher};

use super::ValueRef;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Object {
    properties: HashTrieMap<String, ValueRef>,
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

impl FromIterator<(String, ValueRef)> for Object {
    fn from_iter<T: IntoIterator<Item = (String, ValueRef)>>(iter: T) -> Self {
        Object {
            properties: HashTrieMap::from_iter(iter),
        }
    }
}
