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
}
