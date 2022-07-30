use std::rc::Rc;

use self::{environment::Environment, module::Module};

use super::{
    ast::{expression::Query, fn_def::FnDef, type_expression::TypeDef},
    db::{
        query::{QueryPlanner, QueryResult},
        val, ObjectStorage, Value,
    },
};

pub mod environment;
pub mod module;
pub mod native_fn;

type Identifier = String;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EvalContext {
    env: Environment,
    obj_storage: ObjectStorage,
    query_planner: QueryPlanner,
}

impl EvalContext {
    pub fn new() -> EvalContext {
        EvalContext {
            env: native_fn::std_env(),
            obj_storage: ObjectStorage::new(),
            query_planner: QueryPlanner::new(),
        }
    }

    #[allow(dead_code)]
    pub fn from_env(env: Environment) -> EvalContext {
        EvalContext {
            env,
            obj_storage: ObjectStorage::new(),
            query_planner: QueryPlanner::new(),
        }
    }

    pub fn with_child_env(&self) -> Self {
        EvalContext {
            env: self.env.child(),
            obj_storage: ObjectStorage::new(),
            query_planner: QueryPlanner::new(),
        }
    }

    pub fn define_fn(&mut self, fn_def: FnDef) -> Value {
        let mut module = self.current_module();
        let id = fn_def.id().clone();
        module.fn_def(Rc::new(fn_def));
        val::string(format!("FnDef {}", id))
    }

    pub fn call_function<ID: ToString>(&self, id: ID, args: Vec<Value>) -> Value {
        let id = id.to_string();
        match self.current_module().lookup_fn(&id) {
            Some(func) => {
                let given_argc = args.len();
                let expected_argc = func.args().len();
                if given_argc != expected_argc {
                    panic!(
                        "Function arity mismatch for {}: Expected {} but got {}",
                        id, expected_argc, given_argc
                    );
                }
                let mut fn_ctx = self.with_child_env();
                for (arg_name, arg_val) in func.args().iter().zip(args.clone()) {
                    fn_ctx.define_var(arg_name, arg_val)
                }
                func.body().call(&mut fn_ctx, &args)
            }
            None => {
                panic!("unknown function: {}", id)
            }
        }
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

    pub fn schedule_query(&self, query: &Query) -> QueryResult {
        self.query_planner.query(query)
    }
}

pub trait Evaluate<T> {
    fn evaluate(&self, context: &mut EvalContext) -> T;
}
