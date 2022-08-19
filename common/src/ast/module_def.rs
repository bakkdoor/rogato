use super::{ASTDepth, Identifier};
use std::fmt::Display;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct ModuleDef {
    id: Identifier,
    exports: ModuleExports,
}

impl ModuleDef {
    pub fn new(id: Identifier, exports: ModuleExports) -> ModuleDef {
        ModuleDef { id, exports }
    }

    pub fn id(&self) -> &Identifier {
        &self.id
    }
    pub fn exports(&self) -> &ModuleExports {
        &self.exports
    }
}

impl Display for ModuleDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("module ")?;
        self.id.fmt(f)?;
        f.write_str(" { ")?;
        self.exports.fmt(f)?;
        f.write_str(" }")
    }
}

impl ASTDepth for ModuleDef {
    fn ast_depth(&self) -> usize {
        1 + self.exports.ast_depth()
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct ModuleExports {
    exports: Vec<Identifier>,
}

impl ModuleExports {
    pub fn new(exports: Vec<Identifier>) -> Self {
        ModuleExports { exports }
    }

    pub fn len(&self) -> usize {
        self.exports.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> std::slice::Iter<Identifier> {
        self.exports.iter()
    }
}

impl Display for ModuleExports {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut is_first = true;
        for export in self.exports.iter() {
            if !is_first {
                f.write_str(", ")?;
            }
            export.fmt(f)?;
            is_first = false;
        }
        Ok(())
    }
}

impl ASTDepth for ModuleExports {
    fn ast_depth(&self) -> usize {
        self.exports.len()
    }
}
