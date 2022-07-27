use std::rc::Rc;

use self::{environment::Environment, module::Module};

use super::{
    ast::{fn_def::FnDef, type_expression::TypeDef},
    db::{val, ObjectStorage, Value},
};

pub mod environment;
pub mod module;

type Identifier = String;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EvalContext {
    env: Environment,
    obj_storage: ObjectStorage,
}

impl EvalContext {
    pub fn new() -> EvalContext {
        EvalContext {
            env: Environment::new(),
            obj_storage: ObjectStorage::new(),
        }
    }

    #[allow(dead_code)]
    pub fn from_env(env: Environment) -> EvalContext {
        EvalContext {
            env,
            obj_storage: ObjectStorage::new(),
        }
    }

    pub fn with_child_env(&self) -> Self {
        EvalContext {
            env: self.env.child(),
            obj_storage: ObjectStorage::new(),
        }
    }

    pub fn define_fn(&mut self, fn_def: FnDef) -> Value {
        let mut module = self.current_module();
        let id = fn_def.id();
        module.fn_def(Rc::new(fn_def));
        val::string(format!("FnDef {}", id))
    }

    pub fn call_function(&self, id: &str, args: Vec<Value>) -> Value {
        // TODO
        val::string(format!("call function {}({:?})", id, args))
    }

    pub fn define_var(&mut self, id: &Identifier, val: Value) {
        self.env.define_var(id, val);
    }

    pub fn lookup_var(&self, id: &str) -> Option<Value> {
        self.env.lookup_var(id)
    }

    #[allow(dead_code)]
    pub fn define_module(&mut self, id: &Identifier, module: Module) {
        self.env.define_module(id, module);
    }

    #[allow(dead_code)]
    pub fn lookup_module(&self, id: &Identifier) -> Option<Module> {
        self.env.lookup_module(id)
    }

    pub fn lookup_const(&self, id: &Identifier) -> Option<Value> {
        self.env.lookup_const(id)
    }

    pub fn lookup_type(&self, id: &Identifier) -> Option<Rc<TypeDef>> {
        self.env.lookup_type(id)
    }

    pub fn current_module(&self) -> Module {
        self.env.current_module()
    }
}

pub trait Evaluate<T> {
    fn evaluate(&self, context: &mut EvalContext) -> T;
}
