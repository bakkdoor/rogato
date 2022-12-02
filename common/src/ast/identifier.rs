use std::fmt::Display;
use std::hash::Hash;

use smol_str::SmolStr;
pub type Identifier = SmolStr;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct ModIdentifier(Identifier);

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct FnIdentifier(Identifier);

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct OpIdentifier(Identifier);

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct VarIdentifier(Identifier);

impl VarIdentifier {
    pub fn new<ID: Into<Identifier>>(id: ID) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<VarIdentifier> for Identifier {
    fn from(id: VarIdentifier) -> Self {
        id.0
    }
}

impl From<&VarIdentifier> for Identifier {
    fn from(id: &VarIdentifier) -> Self {
        id.0.clone()
    }
}

impl From<Identifier> for VarIdentifier {
    fn from(id: Identifier) -> Self {
        VarIdentifier(id)
    }
}

impl From<&Identifier> for VarIdentifier {
    fn from(id: &Identifier) -> Self {
        VarIdentifier(id.clone())
    }
}

impl From<&str> for VarIdentifier {
    fn from(id: &str) -> Self {
        VarIdentifier(id.into())
    }
}

impl From<&&str> for VarIdentifier {
    fn from(id: &&str) -> Self {
        VarIdentifier((*id).into())
    }
}

impl Display for ModIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_str())
    }
}

impl Display for FnIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_str())
    }
}

impl Display for OpIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_str())
    }
}

impl Display for VarIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_str())
    }
}
