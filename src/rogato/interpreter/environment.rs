use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::{module::Module, Identifier};
use crate::rogato::{ast::type_expression::TypeDef, db::Value};

#[derive(Clone, PartialEq, Eq, Debug)]
struct State {
    parent: Option<Environment>,
    variables: HashMap<Identifier, Value>,
    modules: HashMap<Identifier, Module>,
    module: Identifier,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Environment {
    state: Rc<RefCell<State>>,
}

impl Environment {
    pub fn new() -> Environment {
        let mut modules = HashMap::new();
        let mod_name = "Std".to_string();
        modules.insert(mod_name.clone(), Module::new(&mod_name));
        let state = State {
            parent: None,
            variables: HashMap::new(),
            modules,
            module: mod_name,
        };
        Environment {
            state: Rc::new(RefCell::new(state)),
        }
    }

    #[allow(dead_code)]
    pub fn new_with_module(module_name: &Identifier) -> Environment {
        let mut modules = HashMap::new();
        modules.insert(module_name.clone(), Module::new(module_name));
        let state = State {
            parent: None,
            variables: HashMap::new(),
            modules,
            module: module_name.clone(),
        };
        Environment {
            state: Rc::new(RefCell::new(state)),
        }
    }

    pub fn child(&self) -> Environment {
        let state = State {
            parent: Some(self.clone()),
            variables: HashMap::new(),
            modules: HashMap::new(),
            module: self.module(),
        };
        Environment {
            state: Rc::new(RefCell::new(state)),
        }
    }

    #[allow(dead_code)]
    pub fn child_with_module(&self, module: &Identifier) -> Environment {
        let mut modules = HashMap::new();
        modules.insert(module.clone(), Module::new(module));
        let state = State {
            parent: Some(self.clone()),
            variables: HashMap::new(),
            modules,
            module: module.clone(),
        };
        Environment {
            state: Rc::new(RefCell::new(state)),
        }
    }

    pub fn module(&self) -> Identifier {
        self.state.borrow().module.clone()
    }

    pub fn define_var(&mut self, id: &Identifier, val: Value) {
        self.state.borrow_mut().variables.insert(id.clone(), val);
    }

    pub fn lookup_var(&self, id: &str) -> Option<Value> {
        let state = self.state.borrow();
        match state.variables.get(id) {
            Some(expr) => Some(expr.clone()),
            None => match &state.parent {
                Some(parent_env) => parent_env.lookup_var(id),
                None => None,
            },
        }
    }

    pub fn define_module(&mut self, id: &Identifier, module: Module) -> &mut Self {
        self.state.borrow_mut().modules.insert(id.clone(), module);
        self
    }

    pub fn current_module(&self) -> Module {
        let state = self.state.borrow();
        match self.lookup_module(&state.module) {
            Some(module) => module,
            None => {
                panic!("No current module found: {}", state.module)
            }
        }
    }

    pub fn lookup_module(&self, id: &Identifier) -> Option<Module> {
        let state = self.state.borrow();
        match state.modules.get(id) {
            Some(module) => Some(module.clone()),
            None => match &state.parent {
                Some(parent_env) => parent_env.lookup_module(id),
                None => None,
            },
        }
    }

    pub fn lookup_const(&self, id: &Identifier) -> Option<Value> {
        let state = self.state.borrow();
        match self.lookup_module(&state.module) {
            Some(module) => module.lookup_const(id),
            None => {
                let err_str = format!(
                    "Module not found: {} while trying to lookup const: {}",
                    state.module, id
                );
                eprintln!("{}", err_str);
                panic!("{}", err_str)
            }
        }
    }

    pub fn lookup_type(&self, id: &Identifier) -> Option<Rc<TypeDef>> {
        let state = self.state.borrow();
        match self.lookup_module(&state.module) {
            Some(module) => module.lookup_type(id),
            None => {
                let err_str = format!(
                    "Module not found: {} while trying to lookup type: {}",
                    state.module, id
                );
                eprintln!("{}", err_str);
                panic!("{}", err_str)
            }
        }
    }
}
