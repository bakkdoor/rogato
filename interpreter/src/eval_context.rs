use rogato_common::{
    ast::{
        fn_def::FnDefBody,
        lambda::{Lambda, LambdaClosureContext, LambdaClosureEvalError},
    },
    native_fn::NativeFnContext,
};

use super::{environment::Environment, module::Module, EvalError, Identifier, ValueRef};
use crate::{
    lib_std,
    query_planner::{QueryPlanner, QueryResult},
    Evaluate,
};
use rogato_common::{
    ast::{expression::Query, fn_def::FnDef, type_expression::TypeDef},
    val,
};
use rogato_db::db::ObjectStorage;
use std::{cell::RefCell, rc::Rc};
use uuid::Uuid;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EvalContext {
    id: Uuid,
    env: Environment,
    obj_storage: ObjectStorage,
    query_planner: QueryPlanner,
}

impl Default for EvalContext {
    fn default() -> Self {
        EvalContext::new()
    }
}

impl EvalContext {
    pub fn new() -> EvalContext {
        EvalContext {
            id: uuid::Uuid::new_v4(),
            env: lib_std::env(),
            obj_storage: ObjectStorage::new(),
            query_planner: QueryPlanner::new(),
        }
    }

    pub fn from_env(env: Environment) -> EvalContext {
        EvalContext {
            id: uuid::Uuid::new_v4(),
            env,
            obj_storage: ObjectStorage::new(),
            query_planner: QueryPlanner::new(),
        }
    }

    pub fn with_child_env(&self) -> Self {
        EvalContext {
            id: uuid::Uuid::new_v4(),
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

    pub fn lookup_fn(&mut self, id: &Identifier) -> Option<Rc<FnDef>> {
        self.env.lookup_fn(id)
    }

    pub fn call_lambda(
        &mut self,
        lambda_ctx: Rc<RefCell<dyn LambdaClosureContext>>,
        lambda: &Lambda,
        args: &[ValueRef],
    ) -> Result<ValueRef, EvalError> {
        lambda_ctx
            .borrow_mut()
            .evaluate_lambda_call(lambda, args)
            .map_err(|e| EvalError::Unknown(e.to_string()))
    }

    pub fn call_function_direct(
        &mut self,
        func: &FnDef,
        args: &[ValueRef],
    ) -> Result<ValueRef, EvalError> {
        let given_argc = args.len();
        let expected_argc = func.args().len();

        if given_argc < func.args().required_args() {
            eprintln!(
                "Function arity mismatch for {}: Expected {} but got {}",
                func.id().clone(),
                expected_argc,
                given_argc
            );
            return Err(EvalError::FunctionArityMismatch(
                func.id().clone(),
                expected_argc,
                given_argc,
            ));
        }

        let mut fn_ctx = self.with_child_env();
        for (arg_name, arg_val) in func.args().iter().zip(args) {
            fn_ctx.define_var(arg_name, Rc::clone(arg_val))
        }

        match &*(func.body()) {
            FnDefBody::NativeFn(f) => f(&fn_ctx, args).map_err(EvalError::from),
            FnDefBody::RogatoFn(expr) => expr.evaluate(&mut fn_ctx),
        }
    }

    pub fn call_function(
        &mut self,
        id: &Identifier,
        args: &[ValueRef],
    ) -> Option<Result<ValueRef, EvalError>> {
        self.lookup_fn(id)
            .map(|func| self.call_function_direct(func.as_ref(), args))
    }

    pub fn define_var(&mut self, id: &Identifier, val: ValueRef) {
        self.env.define_var(id, val);
    }

    pub fn lookup_var(&self, id: &str) -> Option<ValueRef> {
        self.env.lookup_var(id)
    }

    pub fn define_module(&mut self, module: Module) {
        self.env.define_module(module);
    }

    pub fn lookup_module(&self, id: &Identifier) -> Option<Module> {
        self.env.lookup_module(id)
    }

    pub fn lookup_const(&self, id: &Identifier) -> Option<ValueRef> {
        self.env.lookup_const(id)
    }

    pub fn lookup_type(&self, id: &Identifier) -> Option<Rc<TypeDef>> {
        self.env.lookup_type(id)
    }

    pub fn lookup_db_type(&self, id: &Identifier) -> Option<Rc<TypeDef>> {
        // TODO: do lookup / verfication with DB instead
        self.env.lookup_type(id)
    }

    pub fn current_module(&self) -> Module {
        self.env.current_module()
    }

    pub fn schedule_query(&mut self, query: &Query) -> QueryResult {
        let mut eval_ctx = self.with_child_env();
        self.query_planner.query(&mut eval_ctx, query)
    }
}

impl NativeFnContext for EvalContext {
    fn lookup_var(&self, id: rogato_common::ast::Identifier) -> Option<ValueRef> {
        self.lookup_var(id.as_str())
    }

    fn lookup_const(&self, id: &rogato_common::ast::Identifier) -> Option<ValueRef> {
        self.lookup_const(id)
    }

    fn call_function_direct(
        &mut self,
        func: &FnDef,
        args: &[ValueRef],
    ) -> Result<ValueRef, rogato_common::native_fn::NativeFnError> {
        self.call_function_direct(func, args)
            .map_err(|e| rogato_common::native_fn::NativeFnError::EvaluationFailed(e.to_string()))
    }
}

impl LambdaClosureContext for EvalContext {
    fn hash_id(&self) -> String {
        self.id.to_string()
    }

    fn lookup_var(&self, id: Identifier) -> Option<ValueRef> {
        self.lookup_var(id.as_str())
    }

    fn define_var(&mut self, id: &rogato_common::ast::Identifier, val: ValueRef) {
        self.define_var(id, val)
    }

    fn with_child_env(&self) -> Box<dyn LambdaClosureContext> {
        Box::new(self.with_child_env())
    }

    fn evaluate_lambda_call(
        &mut self,
        lambda: &Lambda,
        args: &[ValueRef],
    ) -> Result<ValueRef, LambdaClosureEvalError> {
        let given_argc = args.len();
        let expected_argc = lambda.args().len();
        if given_argc != expected_argc {
            eprintln!(
                "Lambda arity mismatch: Expected {} but got {}",
                expected_argc, given_argc
            );
            return Err(LambdaClosureEvalError::LambdaArityMismatch(
                expected_argc,
                given_argc,
            ));
        }

        let mut call_ctx = self.with_child_env();

        for (arg_id, arg_val) in lambda.args().iter().zip(args.into_iter()) {
            call_ctx.define_var(arg_id, Rc::clone(arg_val))
        }

        lambda.body().evaluate(&mut call_ctx).map_err(|e| {
            rogato_common::ast::lambda::LambdaClosureEvalError::EvaluationFailed(e.to_string())
        })
    }
}
