use std::cell::RefCell;
use std::rc::Rc;
use std::{collections::HashMap, fmt::Display};

use crate::ValueRef;
use rogato_common::ast::{fn_def::FnDef, type_expression::TypeDef};

use super::Identifier;

#[derive(Clone, PartialEq, Eq, Debug)]
struct State {
    id: Identifier,
    fn_defs: HashMap<Identifier, Rc<FnDef>>,
    type_defs: HashMap<Identifier, Rc<TypeDef>>,
    constants: HashMap<Identifier, ValueRef>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Module {
    state: Rc<RefCell<State>>,
}

impl Module {
    pub fn new(id: &str) -> Module {
        let state = State {
            id: id.into(),
            fn_defs: HashMap::new(),
            type_defs: HashMap::new(),
            constants: HashMap::new(),
        };
        Module {
            state: Rc::new(RefCell::new(state)),
        }
    }

    pub fn id(&self) -> Identifier {
        let state = self.state.borrow();
        state.id.clone()
    }

    pub fn fn_def(&mut self, fn_def: Rc<FnDef>) {
        let mut state = self.state.borrow_mut();
        state.fn_defs.insert(fn_def.id().clone(), fn_def);
    }

    #[allow(dead_code)]
    pub fn lookup_fn(&self, id: &Identifier) -> Option<Rc<FnDef>> {
        let state = self.state.borrow();
        let opt = state.fn_defs.get(id).cloned();
        opt
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
    pub fn const_def(&mut self, id: &Identifier, val: ValueRef) {
        let mut state = self.state.borrow_mut();
        state.constants.insert(id.clone(), val);
    }

    pub fn lookup_const(&self, id: &Identifier) -> Option<ValueRef> {
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
