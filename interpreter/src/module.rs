use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::{collections::HashMap, fmt::Display};

use crate::ValueRef;
use rogato_common::ast::fn_def::{FnDefArgs, FnDefBody, FnDefVariant};
use rogato_common::ast::pattern::Pattern;
use rogato_common::ast::{fn_def::FnDef, type_expression::TypeDef};
use rogato_common::native_fn::NativeFn;

use super::Identifier;

#[derive(Clone, PartialEq, Eq, Debug)]
struct State {
    id: Identifier,
    fn_defs: HashMap<Identifier, Rc<RefCell<FnDef>>>,
    type_defs: HashMap<Identifier, Rc<TypeDef>>,
    constants: HashMap<Identifier, ValueRef>,
    exports: HashSet<Identifier>,
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
            exports: HashSet::new(),
        };
        Module {
            state: Rc::new(RefCell::new(state)),
        }
    }

    pub fn id(&self) -> Identifier {
        let state = self.state.borrow();
        state.id.clone()
    }

    pub fn export(&mut self, id: Identifier) {
        let mut state = self.state.borrow_mut();
        state.exports.insert(id);
    }

    pub fn fn_def<ID: Into<Identifier>>(&mut self, id: ID, fn_variant: FnDefVariant) {
        let id: Identifier = id.into();
        if self.has_fn_defined(&id) {
            let (args, body) = fn_variant;
            self.state
                .borrow()
                .fn_defs
                .get(&id)
                .map(|f| f.borrow_mut().variants.add(args, body))
                .unwrap_or_else(|| eprintln!("EvalContext::define_fn_variant failed for: {}", id))
        } else {
            let (args, body) = fn_variant;
            let fn_def = FnDef::new(id.clone(), args, body);
            self.state.borrow_mut().fn_defs.insert(id, fn_def);
        }
    }

    pub fn fn_def_native(&mut self, id: &str, args: &[&str], fn_body: NativeFn) {
        let id: Identifier = id.into();

        let args = FnDefArgs::new(
            args.iter()
                .map(|a| Rc::new(Pattern::Var(a.into())))
                .collect(),
        );
        let body = Rc::new(FnDefBody::native(fn_body));

        self.fn_def(id, (args, body));
    }

    fn has_fn_defined(&self, id: &Identifier) -> bool {
        self.state.borrow().fn_defs.contains_key(id)
    }

    pub fn lookup_fn(&self, id: &Identifier) -> Option<Rc<RefCell<FnDef>>> {
        let state = self.state.borrow();
        state.fn_defs.get(id).cloned()
    }

    pub fn type_def(&mut self, id: Identifier, type_def: Rc<TypeDef>) {
        let mut state = self.state.borrow_mut();
        state.type_defs.insert(id, type_def);
    }

    pub fn lookup_type(&self, id: &Identifier) -> Option<Rc<TypeDef>> {
        let state = self.state.borrow();
        state.type_defs.get(id).cloned()
    }

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
