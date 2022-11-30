use rogato_common::{
    ast::{
        expression::Expression,
        fn_def::{FnDefBody, FnDefVariant},
        lambda::{Lambda, LambdaClosureContext, LambdaClosureEvalError},
    },
    native_fn::{NativeFnContext, NativeFnError},
};

use super::{environment::Environment, module::Module, EvalError, Identifier, ValueRef};
use crate::{
    environment::Imports,
    lib_std,
    pattern_matching::{PatternMatch, PatternMatching, PatternMatchingError},
    query_planner::{QueryPlanner, QueryResult},
    Evaluate,
};
use rogato_common::{
    ast::{expression::Query, fn_def::FnDef, type_expression::TypeDef},
    val,
};
use rogato_db::db::ObjectStorage;
use std::{cell::RefCell, ops::Deref, rc::Rc};
use uuid::Uuid;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EvalContext {
    id: Uuid,
    env: Environment,
    obj_storage: ObjectStorage,
    query_planner: QueryPlanner,
    current_func_id: Option<Identifier>,
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
            current_func_id: None,
        }
    }

    pub fn from_env(env: Environment) -> EvalContext {
        EvalContext {
            id: uuid::Uuid::new_v4(),
            env,
            obj_storage: ObjectStorage::new(),
            query_planner: QueryPlanner::new(),
            current_func_id: None,
        }
    }

    pub fn with_child_env(&self) -> Self {
        EvalContext {
            id: uuid::Uuid::new_v4(),
            env: self.env.child(),
            obj_storage: self.obj_storage.clone(),
            query_planner: self.query_planner.clone(),
            current_func_id: self.current_func_id.clone(),
        }
    }

    pub fn current_func_id(&self) -> Identifier {
        self.current_func_id.clone().unwrap_or_else(|| "N/A".into())
    }

    pub fn import(
        &mut self,
        module_id: &Identifier,
        imports: Imports,
    ) -> Result<ValueRef, EvalError> {
        let module = self
            .lookup_module(module_id)
            .ok_or_else(|| EvalError::ImportFailed(module_id.clone(), imports.import_ids()))?;

        self.env.import(&module, imports);
        Ok(val::none())
    }

    pub fn define_fn(&mut self, id: &Identifier, fn_variant: FnDefVariant) -> ValueRef {
        let mut module = self.current_module();
        module.fn_def(id.clone(), fn_variant);
        val::string(format!("FnDef {}", id))
    }

    fn lookup_fn(&mut self, id: &Identifier) -> Option<Rc<RefCell<FnDef>>> {
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
        func: Rc<RefCell<FnDef>>,
        args: &[ValueRef],
    ) -> Result<ValueRef, EvalError> {
        let func = func.borrow();

        if func.is_tail_recursive() {
            return self.call_tail_recursive_function_direct(func.deref(), args);
        }

        let given_argc = args.len();
        let required_argc = func.required_args();

        if given_argc < required_argc {
            eprintln!(
                "Function arity mismatch for {}: Expected at least {} but got {}",
                func.id().clone(),
                required_argc,
                given_argc
            );
            return Err(EvalError::FunctionArityMismatch(
                func.id().clone(),
                required_argc,
                given_argc,
            ));
        }

        let last_current_func_id = self.current_func_id.clone();
        self.current_func_id = Some(func.id().clone());

        let mut last_attempted_pattern = None;

        for FnDefVariant(arg_patterns, body) in func.variants_iter() {
            let mut fn_ctx = self.with_child_env();
            let mut matched = 0;
            let mut attempted = 0;
            for (arg_pattern, arg_val) in arg_patterns.iter().zip(args) {
                attempted += 1;
                last_attempted_pattern = Some(Rc::clone(arg_pattern));
                match arg_pattern.pattern_match(&mut fn_ctx, ValueRef::clone(arg_val)) {
                    Ok(PatternMatch::Matched(_)) => {
                        matched += 1;
                        continue;
                    }
                    Ok(PatternMatch::TryNextPattern) => {
                        break;
                    }
                    Err(e) => {
                        return Err(e.into());
                    }
                }
            }

            if matched == attempted {
                let return_val = match &**body {
                    FnDefBody::NativeFn(f) => f(&mut fn_ctx, args).map_err(EvalError::from),
                    FnDefBody::RogatoFn(expr) => expr.evaluate(&mut fn_ctx),
                };
                return return_val;
            }
        }

        self.current_func_id = last_current_func_id;

        return Err(EvalError::PatternBindingFailed(
            func.id().clone(),
            PatternMatchingError::NoFnVariantMatched(
                func.id().clone(),
                last_attempted_pattern,
                args.to_vec(),
            ),
        ));
    }

    pub fn call_tail_recursive_function_direct(
        &mut self,
        func: &FnDef,
        args: &[ValueRef],
    ) -> Result<ValueRef, EvalError> {
        let last_current_func_id = self.current_func_id.clone();
        self.current_func_id = Some(func.id().clone());

        let mut last_attempted_pattern = None;

        let mut return_val = None;
        let mut loop_args = Vec::with_capacity(args.len());
        for arg in args.iter() {
            loop_args.push(ValueRef::clone(arg));
        }

        'looping: loop {
            for FnDefVariant(arg_patterns, body) in func.variants_iter() {
                let mut fn_ctx = self.with_child_env();
                let mut matched = 0;
                let mut attempted = 0;
                for (arg_pattern, arg_val) in arg_patterns.iter().zip(loop_args.iter()) {
                    attempted += 1;
                    last_attempted_pattern = Some(Rc::clone(arg_pattern));
                    match arg_pattern.pattern_match(&mut fn_ctx, ValueRef::clone(arg_val)) {
                        Ok(PatternMatch::Matched(_)) => {
                            matched += 1;
                            continue;
                        }
                        Ok(PatternMatch::TryNextPattern) => {
                            break;
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }

                if matched == attempted {
                    match &**body {
                        FnDefBody::NativeFn(f) => {
                            return_val = Some(f(&mut fn_ctx, args).map_err(EvalError::from)?);
                            break 'looping;
                        }
                        FnDefBody::RogatoFn(expr) => match expr.deref() {
                            Expression::FnCall(fn_call) => {
                                if fn_call.id == func.id {
                                    loop_args = Vec::with_capacity(fn_call.args.len());
                                    for arg_expr in fn_call.args.iter() {
                                        loop_args.push(arg_expr.evaluate(&mut fn_ctx)?);
                                    }
                                    continue 'looping;
                                } else {
                                    return_val = Some(expr.evaluate(&mut fn_ctx)?);
                                    break 'looping;
                                }
                            }
                            Expression::Let(let_expr) => match let_expr.body.deref() {
                                Expression::FnCall(fn_call) => {
                                    for (id, expr) in let_expr.bindings.iter() {
                                        let val = expr.evaluate(&mut fn_ctx)?;
                                        self.define_var(id, val);
                                    }
                                    if fn_call.id == func.id {
                                        loop_args = Vec::with_capacity(fn_call.args.len());
                                        for arg_expr in fn_call.args.iter() {
                                            loop_args.push(arg_expr.evaluate(&mut fn_ctx)?);
                                        }
                                        continue 'looping;
                                    } else {
                                        return_val = Some(expr.evaluate(&mut fn_ctx)?);
                                        break 'looping;
                                    }
                                }
                                expr => {
                                    return_val = Some(expr.evaluate(&mut fn_ctx)?);
                                    break 'looping;
                                }
                            },
                            _ => {
                                return_val = Some(expr.evaluate(&mut fn_ctx)?);
                                break 'looping;
                            }
                        },
                    }
                }
            }
            break 'looping;
        }

        self.current_func_id = last_current_func_id;

        if let Some(value) = return_val {
            return Ok(value);
        }

        return Err(EvalError::PatternBindingFailed(
            func.id().clone(),
            PatternMatchingError::NoFnVariantMatched(
                func.id().clone(),
                last_attempted_pattern,
                args.to_vec(),
            ),
        ));
    }

    pub fn call_function(
        &mut self,
        id: &Identifier,
        args: &[ValueRef],
    ) -> Option<Result<ValueRef, EvalError>> {
        self.lookup_fn(id)
            .map(|func| self.call_function_direct(func, args))
    }

    #[inline]
    pub fn define_var(&mut self, id: &Identifier, val: ValueRef) {
        if id.as_str() != "_" {
            self.env.define_var(id, val)
        }
    }

    #[inline]
    pub fn lookup_var(&self, id: &str) -> Option<ValueRef> {
        self.env.lookup_var(id)
    }

    #[inline]
    pub fn define_module(&mut self, module: Module) {
        self.env.define_module(module);
    }

    #[inline]
    pub fn lookup_module(&self, id: &Identifier) -> Option<Module> {
        self.env.lookup_module(id)
    }

    #[inline]
    pub fn lookup_const(&self, id: &Identifier) -> Option<ValueRef> {
        self.env.lookup_const(id)
    }

    #[inline]
    pub fn lookup_type(&self, id: &Identifier) -> Option<Rc<TypeDef>> {
        self.env.lookup_type(id)
    }

    #[inline]
    pub fn lookup_db_type(&self, id: &Identifier) -> Option<Rc<TypeDef>> {
        // TODO: do lookup / verification with DB instead
        self.env.lookup_type(id)
    }

    #[inline]
    pub fn current_module(&self) -> Module {
        self.env.current_module()
    }

    pub fn set_current_module(&mut self, module_id: Identifier) {
        self.env.set_current_module(module_id)
    }

    pub fn schedule_query(&mut self, query: &Query) -> QueryResult {
        let mut eval_ctx = self.with_child_env();
        self.query_planner.query(&mut eval_ctx, query)
    }
}

impl NativeFnContext for EvalContext {
    fn lookup_var(&self, id: &Identifier) -> Option<ValueRef> {
        self.lookup_var(id)
    }

    fn lookup_const(&self, id: &Identifier) -> Option<ValueRef> {
        self.lookup_const(id)
    }

    fn call_function(
        &mut self,
        id: &Identifier,
        args: &[ValueRef],
    ) -> Option<Result<ValueRef, NativeFnError>> {
        self.call_function(id, args)
            .map(|res| res.map_err(|e| NativeFnError::EvaluationFailed(id.clone(), e.to_string())))
    }

    fn call_lambda(
        &mut self,
        lambda_ctx: Rc<RefCell<dyn LambdaClosureContext>>,
        lambda: &Lambda,
        args: &[ValueRef],
    ) -> Result<ValueRef, NativeFnError> {
        self.call_lambda(lambda_ctx, lambda, args)
            .map_err(|e| NativeFnError::Unknown(lambda.to_string().into(), e.to_string()))
    }

    fn call_function_direct(
        &mut self,
        func: Rc<RefCell<FnDef>>,
        args: &[ValueRef],
    ) -> Result<ValueRef, rogato_common::native_fn::NativeFnError> {
        let id = func.borrow().id().clone();
        self.call_function_direct(func, args).map_err(|e| {
            rogato_common::native_fn::NativeFnError::EvaluationFailed(id, e.to_string())
        })
    }
}

impl LambdaClosureContext for EvalContext {
    fn hash_id(&self) -> String {
        self.id.to_string()
    }

    fn lookup_var(&self, id: &Identifier) -> Option<ValueRef> {
        self.lookup_var(id.as_str())
    }

    fn define_var(&mut self, id: &Identifier, val: ValueRef) {
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
        let expected_argc = lambda.args.len();
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

        for (arg_id, arg_val) in lambda.args.iter().zip(args.iter()) {
            call_ctx.define_var(arg_id, ValueRef::clone(arg_val))
        }

        lambda
            .body
            .evaluate(&mut call_ctx)
            .map_err(|e| LambdaClosureEvalError::EvaluationFailed(e.to_string()))
    }
}
