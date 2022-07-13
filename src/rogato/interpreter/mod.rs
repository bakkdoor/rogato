use self::environment::Environment;

use super::db::{val, ObjectStorage, Value};

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

    pub fn env(&'a self) -> &'a Environment<'a> {
        &self.env
    }

    pub fn call_function(&self, id: &str, args: Vec<Value>) -> Value {
        // TODO
        val::string(format!("call function {}({:?})", id, args))
    }
}

pub trait Evaluate<'a, T> {
    fn evaluate(&self, context: &mut EvalContext<'a>) -> T;
}
