use std::cell::RefCell;
use std::rc::Rc;
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

pub type ModuleRef = Rc<RefCell<Module>>;

impl Module {
    #[allow(dead_code)]
    pub fn new(id: &Identifier) -> ModuleRef {
        let module = Module {
            id: id.clone(),
            fn_defs: HashMap::new(),
            type_defs: HashMap::new(),
            constants: HashMap::new(),
        };
        Rc::new(RefCell::new(module))
    }

    #[allow(dead_code)]
    pub fn fn_def(&mut self, fn_def: Box<FnDef>) -> &mut Self {
        self.fn_defs.insert(fn_def.id(), fn_def);
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
    pub fn lookup_type(&self, id: &Identifier) -> Option<Box<TypeDef>> {
        match self.type_defs.get(id) {
            Some(f) => Some(f.clone()),
            None => None,
        }
    }

    #[allow(dead_code)]
    pub fn const_def(&mut self, id: &Identifier, val: Value) {
        self.constants.insert(id.clone(), val);
    }

    #[allow(dead_code)]
    pub fn lookup_const(&self, id: &Identifier) -> Option<Value> {
        match self.constants.get(id) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }
}

impl Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Module {{ id: {:?} }}", self.id))
    }
}
