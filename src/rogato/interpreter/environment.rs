use std::collections::HashMap;

use super::{module::Module, Identifier};
use crate::rogato::db::Value;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Environment<'a> {
    parent: Option<&'a Environment<'a>>,
    variables: HashMap<Identifier, Value>,
    modules: HashMap<Identifier, Module>,
}

impl<'a> Environment<'a> {
    #[allow(dead_code)]
    pub fn new() -> Environment<'a> {
        Environment {
            parent: None,
            variables: HashMap::new(),
            modules: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn child(&'a self) -> Environment<'a> {
        Environment {
            parent: Some(self),
            variables: HashMap::new(),
            modules: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn define_var(&mut self, id: &Identifier, val: Value) {
        self.variables.insert(id.clone(), val);
    }

    #[allow(dead_code)]
    pub fn lookup_var(&'a self, id: &str) -> Option<&'a Value> {
        match self.variables.get(id) {
            Some(expr) => Some(expr),
            None => match self.parent {
                Some(parent_env) => parent_env.lookup_var(id),
                None => None,
            },
        }
    }

    #[allow(dead_code)]
    pub fn define_module(&'a mut self, id: &Identifier, module: Module) -> &'a mut Self {
        self.modules.insert(id.clone(), module);
        self
    }

    #[allow(dead_code)]
    pub fn lookup_module(&'a self, id: &Identifier) -> Option<&'a Module> {
        match self.modules.get(id) {
            Some(module) => Some(module),
            None => match self.parent {
                Some(parent_env) => parent_env.lookup_module(id),
                None => None,
            },
        }
    }
}
