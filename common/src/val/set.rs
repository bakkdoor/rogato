use std::{
    fmt::Display,
    hash::{Hash, Hasher},
};

use super::ValueRef;
use crate::ast::ASTDepth;

#[derive(Clone, Eq, Debug)]
pub struct Set {
    entries: rpds::HashTrieSet<ValueRef>,
}

impl PartialEq for Set {
    fn eq(&self, other: &Self) -> bool {
        self.entries.eq(&other.entries)
    }
}

impl Hash for Set {
    fn hash<H: Hasher>(&self, h: &mut H) {
        let pairs: Vec<_> = self.entries.iter().collect();
        Hash::hash(&pairs, h);
    }
}

impl ASTDepth for Set {
    fn ast_depth(&self) -> usize {
        1 + self.entries.size()
    }
}

impl Display for Set {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Set{{ {} }}", self.entries))
    }
}
