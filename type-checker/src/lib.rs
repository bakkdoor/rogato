use std::{collections::HashMap, rc::Rc};

use rogato_common::ast::{type_expression::TypeDef, ASTId};

#[cfg(test)]
pub mod tests;

pub struct TypeCheckContext {
    types: HashMap<ASTId, Rc<TypeDef>>,
}

impl TypeCheckContext {
    pub fn new() -> Self {
        TypeCheckContext {
            types: HashMap::new(),
        }
    }

    pub fn set_type(&mut self, id: ASTId, type_def: Rc<TypeDef>) -> Option<ASTId> {
        if self.types.contains_key(&id) {
            return None;
        }
        self.types.insert(id, type_def);
        Some(id)
    }

    pub fn lookup_type(&mut self, id: ASTId) -> Option<Rc<TypeDef>> {
        self.types.get(&id).map(Rc::clone)
    }
}

impl Default for TypeCheckContext {
    fn default() -> Self {
        TypeCheckContext::new()
    }
}

use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum TypeCheckError {
    #[error("Unknown type checking error: {0}")]
    Unknown(String),
}

pub trait TypeCheck<T> {
    fn type_check(&self, context: &mut TypeCheckContext) -> Result<T, TypeCheckError>;
}
