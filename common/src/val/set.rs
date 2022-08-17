use std::{
    fmt::Display,
    hash::{Hash, Hasher},
};

use super::ValueRef;
use crate::ast::ASTDepth;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Set {
    entries: rpds::HashTrieSet<ValueRef>,
}

impl ASTDepth for Set {
    fn ast_depth(&self) -> usize {
        1 + self.entries.size()
    }
}

impl Hash for Set {
    fn hash<H: Hasher>(&self, h: &mut H) {
        let pairs: Vec<_> = self.entries.iter().collect();
        Hash::hash(&pairs, h);
    }
}

impl Display for Set {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Set{{ {} }}", self.entries))
    }
}
