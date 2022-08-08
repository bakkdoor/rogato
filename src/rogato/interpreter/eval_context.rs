use super::{environment::Environment, module::Module, native_fn, EvalError, Identifier};
use crate::rogato::{
    ast::{expression::Query, fn_def::FnDef, type_expression::TypeDef},
    db::{
        query::{QueryPlanner, QueryResult},
        val, ObjectStorage, ValueRef,
    },
};
use std::rc::Rc;

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
            obj_storage: self.obj_storage.clone(),
            query_planner: self.query_planner.clone(),
        }
    }

    pub fn define_fn(&mut self, fn_def: Rc<FnDef>) -> ValueRef {
        let mut module = self.current_module();
        let id = fn_def.id().clone();
        module.fn_def(fn_def);
        val::string(format!("FnDef {}", id))
    }

    pub fn call_function(
        &self,
        id: &Identifier,
        args: &[ValueRef],
    ) -> Option<Result<ValueRef, EvalError>> {
        self.current_module().lookup_fn(id).map(|func| {
            let given_argc = args.len();
            let expected_argc = func.args().len();
            if given_argc != expected_argc {
                eprintln!(
                    "Function arity mismatch for {}: Expected {} but got {}",
                    id, expected_argc, given_argc
                );
                return Err(EvalError::FunctionArityMismatch(
                    id.clone(),
                    expected_argc,
                    given_argc,
                ));
            }
            let mut fn_ctx = self.with_child_env();
            for (arg_name, arg_val) in func.args().iter().zip(args) {
                fn_ctx.define_var(arg_name, arg_val.clone())
            }
            func.body().call(&mut fn_ctx, args)
        })
    }

    pub fn define_var(&mut self, id: &Identifier, val: ValueRef) {
        self.env.define_var(id, val);
    }

    pub fn lookup_var(&self, id: &str) -> Option<ValueRef> {
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

    pub fn lookup_const(&self, id: &Identifier) -> Option<ValueRef> {
        self.env.lookup_const(id)
    }

    pub fn lookup_type(&self, id: &Identifier) -> Option<Rc<TypeDef>> {
        self.env.lookup_type(id)
    }

    pub fn current_module(&self) -> Module {
        self.env.current_module()
    }

    pub fn schedule_query(&mut self, query: &Query) -> QueryResult {
        let mut ctx = self.with_child_env();
        self.query_planner.query(&mut ctx, query)
    }
}
