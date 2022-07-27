use std::cell::RefCell;
use std::rc::Rc;
use std::{collections::HashMap, fmt::Display};

use crate::rogato::ast::{fn_def::FnDef, type_expression::TypeDef};
use crate::rogato::db::Value;

use super::Identifier;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ModuleState {
    id: Identifier,
    fn_defs: HashMap<Identifier, Rc<FnDef>>,
    type_defs: HashMap<Identifier, Rc<TypeDef>>,
    constants: HashMap<Identifier, Value>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Module {
    state: Rc<RefCell<ModuleState>>,
}

impl Module {
    pub fn new(id: &Identifier) -> Module {
        let state = ModuleState {
            id: id.clone(),
            fn_defs: HashMap::new(),
            type_defs: HashMap::new(),
            constants: HashMap::new(),
        };
        Module {
            state: Rc::new(RefCell::new(state)),
        }
    }

    pub fn fn_def(&mut self, fn_def: Rc<FnDef>) {
        let mut state = self.state.borrow_mut();
        state.fn_defs.insert(fn_def.id(), fn_def);
    }

    #[allow(dead_code)]
    pub fn lookup_fn(&self, id: &Identifier) -> Option<Rc<FnDef>> {
        let state = self.state.borrow();
        state.fn_defs.get(id).cloned()
    }

    #[allow(dead_code)]
    pub fn type_def(&mut self, id: Identifier, type_def: Rc<TypeDef>) {
        let mut state = self.state.borrow_mut();
        state.type_defs.insert(id, type_def);
    }

    pub fn lookup_type(&self, id: &Identifier) -> Option<Rc<TypeDef>> {
        let state = self.state.borrow();
        state.type_defs.get(id).cloned()
    }

    #[allow(dead_code)]
    pub fn const_def(&mut self, id: &Identifier, val: Value) {
        let mut state = self.state.borrow_mut();
        state.constants.insert(id.clone(), val);
    }

    pub fn lookup_const(&self, id: &Identifier) -> Option<Value> {
        let state = self.state.borrow();
        state.constants.get(id).cloned()
    }
}

impl Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = self.state.borrow();
        f.write_fmt(format_args!("Module {{ id: {:?} }}", state.id))
    }
}
