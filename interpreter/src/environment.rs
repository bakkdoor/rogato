use super::{module::Module, Identifier, ValueRef};
use rogato_common::ast::{fn_def::FnDef, type_expression::TypeDef};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

type FuncId = Identifier;
type TypeId = Identifier;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ImportedIdentifier {
    Func(FuncId),
    Type(TypeId),
    AliasedFunc(FuncId, FuncId), // imported_id, local_alias
    AliasedType(TypeId, TypeId), // imported_id, local_alias
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Imports {
    All,
    Specific(Vec<ImportedIdentifier>),
}

type ImportedModules = HashMap<Identifier, Imports>;

#[derive(Clone, PartialEq, Eq, Debug)]
struct State {
    parent: Option<Environment>,
    variables: HashMap<Identifier, ValueRef>,
    modules: Rc<RefCell<HashMap<Identifier, Module>>>,
    imported_modules: ImportedModules,
    current_module_name: Identifier,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Environment {
    state: Rc<RefCell<State>>,
}

impl Default for Environment {
    fn default() -> Self {
        Environment::new()
    }
}

impl Environment {
    #[allow(dead_code)]
    pub fn new() -> Environment {
        let modules = Rc::new(RefCell::new(HashMap::new()));
        let mod_name: Identifier = "Std".into();
        modules
            .borrow_mut()
            .insert(mod_name.clone(), Module::new(&mod_name));

        let state = State {
            parent: None,
            variables: HashMap::new(),
            modules,
            imported_modules: HashMap::new(),
            current_module_name: mod_name,
        };
        Environment {
            state: Rc::new(RefCell::new(state)),
        }
    }

    #[allow(dead_code)]
    pub fn new_with_imported_modules(imported_modules: ImportedModules) -> Environment {
        let modules = Rc::new(RefCell::new(HashMap::new()));
        let mod_name: Identifier = "Std".into();
        modules
            .borrow_mut()
            .insert(mod_name.clone(), Module::new(&mod_name));

        let state = State {
            parent: None,
            variables: HashMap::new(),
            modules,
            imported_modules,
            current_module_name: mod_name,
        };
        Environment {
            state: Rc::new(RefCell::new(state)),
        }
    }

    pub fn child(&self) -> Environment {
        let curr_state = self.state.borrow();
        let state = State {
            parent: Some(self.clone()),
            variables: HashMap::new(),
            modules: Rc::clone(&curr_state.modules),
            imported_modules: self.imported_modules(),
            current_module_name: curr_state.current_module_name.clone(),
        };
        Environment {
            state: Rc::new(RefCell::new(state)),
        }
    }

    #[allow(dead_code)]
    pub fn child_with_imported_modules(&self, imported_modules: ImportedModules) -> Environment {
        let curr_state = self.state.borrow();
        let state = State {
            parent: Some(self.clone()),
            variables: HashMap::new(),
            modules: Rc::clone(&curr_state.modules),
            imported_modules,
            current_module_name: curr_state.current_module_name.clone(),
        };
        Environment {
            state: Rc::new(RefCell::new(state)),
        }
    }

    pub fn imported_modules(&self) -> ImportedModules {
        self.state.borrow().imported_modules.clone()
    }

    pub fn define_var(&mut self, id: &Identifier, val: ValueRef) {
        self.state.borrow_mut().variables.insert(id.clone(), val);
    }

    pub fn lookup_var(&self, id: &str) -> Option<ValueRef> {
        let state = self.state.borrow();
        let res = match state.variables.get(id) {
            Some(expr) => Some(expr.clone()),
            None => match &state.parent {
                Some(parent_env) => parent_env.lookup_var(id),
                None => None,
            },
        };
        res
    }

    pub fn define_module(&mut self, module: Module) {
        let id = module.id();
        let state = self.state.borrow_mut();
        state.modules.borrow_mut().insert(id, module);
    }

    pub fn current_module(&self) -> Module {
        let state = self.state.borrow();
        self.lookup_module(&state.current_module_name)
            .expect("current_module should be set")
    }

    pub fn lookup_module(&self, id: &Identifier) -> Option<Module> {
        match self.state.borrow().modules.borrow().get(id) {
            Some(module) => Some(module.clone()),
            None => match &self.state.borrow().parent {
                Some(parent_env) => parent_env.lookup_module(id),
                None => None,
            },
        }
    }

    pub fn lookup_const(&self, id: &Identifier) -> Option<ValueRef> {
        match self
            .lookup_module_for_const(id)
            .and_then(|m| m.lookup_const(id))
        {
            Some(val) => Some(val),
            None => {
                let state = self.state.borrow();
                match &state.parent {
                    Some(parent_env) => parent_env.lookup_const(id),
                    None => None,
                }
            }
        }
    }

    pub fn lookup_type(&self, id: &Identifier) -> Option<Rc<TypeDef>> {
        match self
            .lookup_module_for_type(id)
            .and_then(|m| m.lookup_type(id))
        {
            Some(type_) => Some(type_),
            None => {
                let state = self.state.borrow();
                match &state.parent {
                    Some(parent_env) => parent_env.lookup_type(id),
                    None => None,
                }
            }
        }
    }

    pub fn lookup_fn(&self, id: &Identifier) -> Option<Rc<FnDef>> {
        let curr_mod = self.current_module();
        match curr_mod.lookup_fn(id) {
            Some(fn_def) => Some(fn_def),
            None => {
                let state = self.state.borrow();
                match &state.parent {
                    Some(parent_env) => parent_env.lookup_fn(id),
                    None => None,
                }
            }
        }
    }

    pub fn lookup_module_for_fn(&self, id: &Identifier) -> Option<Module> {
        let curr_mod = self.current_module();
        match curr_mod.lookup_type(id) {
            Some(_) => Some(curr_mod),
            None => {
                let state = self.state.borrow();
                for (module_id, imports) in state.imported_modules.iter() {
                    match imports {
                        Imports::All => match self.lookup_module(module_id) {
                            Some(module) => match module.lookup_fn(id) {
                                Some(_) => return Some(module),
                                None => continue,
                            },
                            None => continue,
                        },
                        Imports::Specific(imported_ids) => {
                            for iid in imported_ids.iter() {
                                if import_id_matches(iid, id) {
                                    return Some(self.lookup_module(module_id).unwrap());
                                }
                            }
                        }
                    }
                }

                None
            }
        }
    }

    pub fn lookup_module_for_type(&self, id: &Identifier) -> Option<Module> {
        let curr_mod = self.current_module();
        match curr_mod.lookup_type(id) {
            Some(_) => Some(curr_mod),
            None => {
                let state = self.state.borrow();
                for (module_id, imports) in state.imported_modules.iter() {
                    match imports {
                        Imports::All => match self.lookup_module(module_id) {
                            Some(module) => match module.lookup_type(id) {
                                Some(_) => return Some(module),
                                None => continue,
                            },
                            None => continue,
                        },
                        Imports::Specific(imported_ids) => {
                            for iid in imported_ids.iter() {
                                if import_id_matches(iid, id) {
                                    return Some(self.lookup_module(module_id).unwrap());
                                }
                            }
                        }
                    }
                }

                None
            }
        }
    }

    pub fn lookup_module_for_const(&self, id: &Identifier) -> Option<Module> {
        let curr_mod = self.current_module();
        match curr_mod.lookup_type(id) {
            Some(_) => Some(curr_mod),
            None => {
                let state = self.state.borrow();
                for (module_id, imports) in state.imported_modules.iter() {
                    match imports {
                        Imports::All => match self.lookup_module(module_id) {
                            Some(module) => match module.lookup_const(id) {
                                Some(_) => return Some(module),
                                None => continue,
                            },
                            None => continue,
                        },
                        Imports::Specific(imported_ids) => {
                            for iid in imported_ids.iter() {
                                if import_id_matches(iid, id) {
                                    return Some(self.lookup_module(module_id).unwrap());
                                }
                            }
                        }
                    }
                }

                None
            }
        }
    }
}

fn import_id_matches(imported_id: &ImportedIdentifier, id: &Identifier) -> bool {
    match imported_id {
        ImportedIdentifier::Func(func_id) => func_id == id,
        ImportedIdentifier::Type(type_id) => type_id == id,
        ImportedIdentifier::AliasedFunc(_import_id, alias_id) => alias_id == id,
        ImportedIdentifier::AliasedType(_import_id, alias_id) => alias_id == id,
    }
}
