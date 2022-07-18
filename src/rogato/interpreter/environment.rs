use std::collections::HashMap;

use super::{module::Module, Identifier};
use crate::rogato::{ast::type_expression::TypeDef, db::Value};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Environment<'a> {
    parent: Option<&'a Environment<'a>>,
    variables: HashMap<Identifier, Value>,
    modules: HashMap<Identifier, Module>,
    module: Identifier,
}

impl<'a> Environment<'a> {
    #[allow(dead_code)]
    pub fn new<'b>() -> Environment<'b> {
        let mut modules = HashMap::new();
        let mod_name = "Std".to_string();
        modules.insert(mod_name.clone(), Module::new(&mod_name));

        Environment {
            parent: None,
            variables: HashMap::new(),
            modules,
            module: mod_name,
        }
    }

    #[allow(dead_code)]
    pub fn new_with_module(module: &Identifier) -> Environment<'_> {
        Environment {
            parent: None,
            variables: HashMap::new(),
            modules: HashMap::new(),
            module: module.clone(),
        }
    }

    #[allow(dead_code)]
    pub fn child(&'a self) -> Environment<'a> {
        Environment {
            parent: Some(self),
            variables: HashMap::new(),
            modules: HashMap::new(),
            module: self.module.clone(),
        }
    }

    #[allow(dead_code)]
    pub fn child_with_module(&'a self, module: &Identifier) -> Environment<'a> {
        let mut modules = HashMap::new();
        modules.insert(module.clone(), Module::new(module));
        Environment {
            parent: Some(self),
            variables: HashMap::new(),
            modules,
            module: module.clone(),
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

    #[allow(dead_code)]
    pub fn lookup_const(&'a self, id: &Identifier) -> Option<&'a Value> {
        match self.lookup_module(&self.module) {
            Some(module) => module.lookup_const(id),
            None => {
                let err_str = format!(
                    "Module not found: {} while trying to lookup const: {}",
                    self.module, id
                );
                eprintln!("{}", err_str);
                panic!("{}", err_str)
            }
        }
    }

    #[allow(dead_code)]
    pub fn lookup_type(&'a self, id: &Identifier) -> Option<&'a TypeDef> {
        match self.lookup_module(&self.module) {
            Some(module) => module.lookup_type(id),
            None => {
                let err_str = format!(
                    "Module not found: {} while trying to lookup type: {}",
                    self.module, id
                );
                eprintln!("{}", err_str);
                panic!("{}", err_str)
            }
        }
    }
}
