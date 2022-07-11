use std::collections::HashMap;

use super::{module::Module, Identifier};
use crate::rogato::ast::expression::Expression;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Environment<'a> {
    parent: Option<&'a Environment<'a>>,
    variables: HashMap<Identifier, Box<Expression>>,
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
            parent: Some(&self),
            variables: HashMap::new(),
            modules: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn define_var(&'a mut self, id: &Identifier, val: Box<Expression>) -> &'a mut Self {
        self.variables.insert(id.clone(), val);
        self
    }

    #[allow(dead_code)]
    pub fn lookup_var(&'a self, id: &Identifier) -> Option<&'a Expression> {
        match self.variables.get(id) {
            Some(expr) => Some(expr),
            None => match self.parent {
                Some(parent_env) => parent_env.lookup_var(id),
                None => None,
            },
        }
    }
}
