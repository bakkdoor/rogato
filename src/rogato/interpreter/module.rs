use std::{collections::HashMap, fmt::Display};

use crate::rogato::ast::{fn_def::FnDef, type_expression::TypeDef};
use crate::rogato::db::Value;

use super::Identifier;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Module {
    id: Identifier,
    fn_defs: HashMap<Identifier, Box<FnDef>>,
    type_defs: HashMap<Identifier, Box<TypeDef>>,
    constants: HashMap<Identifier, Value>,
}

impl Module {
    #[allow(dead_code)]
    pub fn new(id: &Identifier) -> Module {
        Module {
            id: id.clone(),
            fn_defs: HashMap::new(),
            type_defs: HashMap::new(),
            constants: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn fn_def(&mut self, id: Identifier, fn_def: Box<FnDef>) -> &mut Self {
        self.fn_defs.insert(id, fn_def);
        self
    }

    #[allow(dead_code)]
    pub fn lookup_fn<'a>(&'a self, id: &Identifier) -> Option<&'a FnDef> {
        match self.fn_defs.get(id) {
            Some(f) => Some(f),
            None => None,
        }
    }

    #[allow(dead_code)]
    pub fn type_def(&mut self, id: Identifier, type_def: Box<TypeDef>) -> &mut Self {
        self.type_defs.insert(id, type_def);
        self
    }

    #[allow(dead_code)]
    pub fn lookup_type<'a>(&'a self, id: &Identifier) -> Option<&'a TypeDef> {
        match self.type_defs.get(id) {
            Some(f) => Some(f),
            None => None,
        }
    }

    #[allow(dead_code)]
    pub fn lookup_const<'a>(&'a self, id: &Identifier) -> Option<&'a Value> {
        match self.constants.get(id) {
            Some(v) => Some(v),
            None => None,
        }
    }

    #[allow(dead_code)]
    pub fn define_const<'a>(&'a mut self, id: &Identifier, val: Value) {
        self.constants.insert(id.clone(), val);
    }
}

impl Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Module {{ id: {:?} }}", self.id))
    }
}
