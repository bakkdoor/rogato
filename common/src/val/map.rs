use rpds::HashTrieMap;
use std::hash::{Hash, Hasher};

use crate::ast::ASTDepth;

use super::ValueRef;

#[derive(Clone, Eq, Debug)]
pub struct Map {
    entries: HashTrieMap<String, ValueRef>,
}

impl PartialEq for Map {
    fn eq(&self, other: &Self) -> bool {
        self.entries.eq(&other.entries)
    }
}

impl Hash for Map {
    fn hash<H: Hasher>(&self, h: &mut H) {
        let mut pairs: Vec<_> = self.entries.iter().collect();
        pairs.sort_by_key(|i| i.0);
        Hash::hash(&pairs, h);
    }
}

impl ASTDepth for Map {
    fn ast_depth(&self) -> usize {
        self.entries.size()
    }
}
