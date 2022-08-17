use std::{
    fmt::Display,
    hash::{Hash, Hasher},
};

use super::ValueRef;
use crate::ast::ASTDepth;

#[derive(Clone, Eq, Debug)]
pub struct Queue {
    entries: rpds::Queue<ValueRef>,
}

impl PartialEq for Queue {
    fn eq(&self, other: &Self) -> bool {
        self.entries.eq(&other.entries)
    }
}

impl Hash for Queue {
    fn hash<H: Hasher>(&self, h: &mut H) {
        let pairs: Vec<_> = self.entries.iter().collect();
        Hash::hash(&pairs, h);
    }
}

impl ASTDepth for Queue {
    fn ast_depth(&self) -> usize {
        1 + self.entries.len()
    }
}

impl Display for Queue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Queue{{ {} }}", self.entries))
    }
}
