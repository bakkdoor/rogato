use self::{environment::Environment, module::Module};

use super::{
    ast::type_expression::TypeDef,
    db::{val, ObjectStorage, Value},
};

pub mod environment;
pub mod module;

type Identifier = String;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EvalContext<'a> {
    env: Environment<'a>,
    obj_storage: ObjectStorage,
}

impl<'a> EvalContext<'a> {
    #[allow(dead_code)]
    pub fn new() -> EvalContext<'a> {
        EvalContext {
            env: Environment::new(),
            obj_storage: ObjectStorage::new(),
        }
    }

    #[allow(dead_code)]
    pub fn from_env(env: &Environment<'a>) -> EvalContext<'a> {
        EvalContext {
            env: env.clone(),
            obj_storage: ObjectStorage::new(),
        }
    }

    pub fn call_function(&self, id: &str, args: Vec<Value>) -> Value {
        // TODO
        val::string(format!("call function {}({:?})", id, args))
    }

    pub fn define_var(&'_ mut self, id: &Identifier, val: Value) {
        self.env.define_var(id, val);
    }

    pub fn lookup_var(&'a self, id: &str) -> Option<&'a Value> {
        self.env.lookup_var(id)
    }

    #[allow(dead_code)]
    pub fn define_module(&'a mut self, id: &Identifier, module: Module) {
        self.env.define_module(id, module);
    }

    #[allow(dead_code)]
    pub fn lookup_module(&self, id: &Identifier) -> Option<&Module> {
        self.env.lookup_module(id)
    }

    pub fn lookup_const(&'a self, id: &Identifier) -> Option<&'a Value> {
        self.env.lookup_const(id)
    }

    pub fn lookup_type(&'a self, id: &Identifier) -> Option<&'a TypeDef> {
        self.env.lookup_type(id)
    }

    pub fn with_child_env(&'a self) -> Self {
        EvalContext {
            env: self.env.child(),
            obj_storage: ObjectStorage::new(),
        }
    }
}

pub trait Evaluate<'a, T> {
    fn evaluate(&self, context: &mut EvalContext<'a>) -> T;
}
