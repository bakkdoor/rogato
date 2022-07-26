use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::{
    module::{Module, ModuleRef},
    Identifier,
};
use crate::rogato::{ast::type_expression::TypeDef, db::Value};

pub type EnvironmentRef = Rc<RefCell<Environment>>;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Environment {
    parent: Option<EnvironmentRef>,
    variables: HashMap<Identifier, Value>,
    modules: HashMap<Identifier, ModuleRef>,
    module: Identifier,
}

impl Environment {
    #[allow(dead_code)]
    pub fn new() -> EnvironmentRef {
        let mut modules = HashMap::new();
        let mod_name = "Std".to_string();
        modules.insert(mod_name.clone(), Module::new(&mod_name));

        let env = Environment {
            parent: None,
            variables: HashMap::new(),
            modules,
            module: mod_name,
        };
        Rc::new(RefCell::new(env))
    }

    #[allow(dead_code)]
    pub fn new_with_module(module: &Identifier) -> EnvironmentRef {
        let env = Environment {
            parent: None,
            variables: HashMap::new(),
            modules: HashMap::new(),
            module: module.clone(),
        };
        Rc::new(RefCell::new(env))
    }

    pub fn spawn_child(parent: EnvironmentRef) -> EnvironmentRef {
        let child = Environment {
            parent: Some(parent.clone()),
            variables: HashMap::new(),
            modules: HashMap::new(),
            module: parent.borrow().module().clone(),
        };

        Rc::new(RefCell::new(child))
    }

    #[allow(dead_code)]
    pub fn spawn_child_with_module(module: &Identifier, parent: EnvironmentRef) -> EnvironmentRef {
        let mut modules = HashMap::new();
        modules.insert(module.clone(), Module::new(module));
        let child = Environment {
            parent: Some(parent),
            variables: HashMap::new(),
            modules,
            module: module.clone(),
        };
        Rc::new(RefCell::new(child))
    }

    pub fn module(&self) -> &Identifier {
        &self.module
    }

    #[allow(dead_code)]
    pub fn define_var(&mut self, id: &Identifier, val: Value) {
        self.variables.insert(id.clone(), val);
    }

    #[allow(dead_code)]
    pub fn lookup_var(&self, id: &str) -> Option<Value> {
        match self.variables.get(id) {
            Some(expr) => Some(expr.clone()),
            None => match &self.parent {
                Some(parent_env) => parent_env.borrow_mut().lookup_var(id),
                None => None,
            },
        }
    }

    #[allow(dead_code)]
    pub fn define_module(&mut self, id: &Identifier, module: ModuleRef) -> &mut Self {
        self.modules.insert(id.clone(), module);
        self
    }

    pub fn current_module(&self) -> ModuleRef {
        match self.lookup_module(&self.module) {
            Some(module) => module,
            None => {
                panic!("No current module found: {}", self.module)
            }
        }
    }

    #[allow(dead_code)]
    pub fn lookup_module(&self, id: &Identifier) -> Option<ModuleRef> {
        match self.modules.get(id) {
            Some(module) => Some(module.clone()),
            None => match &self.parent {
                Some(parent_env) => parent_env.borrow_mut().lookup_module(id),
                None => None,
            },
        }
    }

    #[allow(dead_code)]
    pub fn lookup_const(&self, id: &Identifier) -> Option<Value> {
        match self.lookup_module(&self.module) {
            Some(module) => module.borrow().lookup_const(id),
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
    pub fn lookup_type(&self, id: &Identifier) -> Option<Box<TypeDef>> {
        match self.lookup_module(&self.module) {
            Some(module) => module.borrow().lookup_type(id),
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
