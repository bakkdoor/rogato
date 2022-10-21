use inkwell::{
    builder::Builder,
    context::Context,
    execution_engine::ExecutionEngine,
    module::Module,
    passes::PassManager,
    types::{BasicMetadataTypeEnum, BasicType},
    values::{BasicMetadataValueEnum, FloatValue, FunctionValue, PointerValue},
    OptimizationLevel,
};
use rogato_common::{
    ast::{
        expression::Expression,
        fn_call::FnCallArgs,
        fn_def::{FnDef, FnDefBody},
        literal::Literal,
        module_def::ModuleDef,
        type_expression::TypeDef,
        Identifier, Program, AST,
    },
    val,
};
use std::collections::HashMap;

use crate::error::CompileError;

pub type CompileResult<T> = Result<T, CompileError>;

#[derive(Debug)]
pub struct Compiler<'a, 'ctx> {
    context: &'ctx Context,
    module: &'a Module<'ctx>,
    builder: &'a Builder<'ctx>,
    fpm: &'a PassManager<FunctionValue<'ctx>>,
    #[allow(dead_code)]
    execution_engine: &'a ExecutionEngine<'ctx>,

    current_fn_value: Option<FunctionValue<'ctx>>,
    variables: HashMap<String, PointerValue<'ctx>>,
}

impl<'a, 'ctx> Compiler<'a, 'ctx> {
    pub fn new(
        context: &'ctx Context,
        module: &'a Module<'ctx>,
        builder: &'a Builder<'ctx>,
        fpm: &'a PassManager<FunctionValue<'ctx>>,
        execution_engine: &'a ExecutionEngine<'ctx>,
    ) -> Self {
        Self {
            context,
            module,
            builder,
            fpm,
            execution_engine,
            current_fn_value: None,
            variables: HashMap::new(),
        }
    }

    pub fn default_function_pass_manager(
        module: &Module<'ctx>,
    ) -> PassManager<FunctionValue<'ctx>> {
        // Create FPM
        let fpm = PassManager::create(module);

        fpm.add_instruction_combining_pass();
        fpm.add_reassociate_pass();
        fpm.add_gvn_pass();
        fpm.add_cfg_simplification_pass();
        fpm.add_basic_alias_analysis_pass();
        fpm.add_promote_memory_to_register_pass();
        fpm.add_instruction_combining_pass();
        fpm.add_reassociate_pass();
        fpm.initialize();
        fpm
    }

    pub fn context(&self) -> &'ctx Context {
        self.context
    }
    pub fn execution_engine(&self) -> &ExecutionEngine<'ctx> {
        self.execution_engine
    }

    pub fn default_execution_engine(module: &'a Module<'ctx>) -> ExecutionEngine<'ctx> {
        module
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap()
    }

    pub fn new_context() -> Context {
        Context::create()
    }

    pub fn new_module(&self, name: &str) -> Module<'ctx> {
        self.context.create_module(name)
    }

    pub fn lookup_var<S: ToString>(&self, name: S) -> Option<&PointerValue<'ctx>> {
        self.variables.get(&name.to_string())
    }

    pub fn store_var<S: ToString>(&mut self, name: S, pointer_val: PointerValue<'ctx>) {
        self.variables.insert(name.to_string(), pointer_val);
    }

    fn create_entry_block_alloca<T: BasicType<'ctx>>(
        &self,
        ty: T,
        name: &str,
    ) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();

        let entry = self.current_fn_value().get_first_basic_block().unwrap();

        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }

        builder.build_alloca(ty, name)
    }

    /// Returns the `FunctionValue` representing the function being compiled.
    #[inline]
    fn current_fn_value(&self) -> FunctionValue<'ctx> {
        self.current_fn_value.unwrap()
    }

    /// Sets the `FunctionValue` representing the function being compiled.
    #[inline]
    fn set_current_fn_value(&mut self, fn_val: FunctionValue<'ctx>) {
        self.current_fn_value = Some(fn_val)
    }

    #[inline]
    fn clear_current_fn(&mut self) {
        self.current_fn_value = None;
        self.builder.clear_insertion_position();
    }

    /// Gets a defined function given its name.
    #[inline]
    fn get_function(&self, name: &str) -> Option<FunctionValue<'ctx>> {
        self.module.get_function(name)
    }

    fn unknown_error<T, S: ToString>(&self, message: S) -> CompileResult<T> {
        Err(CompileError::Unknown(message.to_string()))
    }

    pub fn not_yed_implemented_error<T, S: ToString>(&self, message: S) -> CompileResult<T> {
        Err(CompileError::NotYetImplemented(message.to_string()))
    }

    pub fn compile_fn_def(&mut self, fn_def: &FnDef) -> CompileResult<FunctionValue<'ctx>> {
        let f32_type = self.context.f32_type();

        let fn_arg_types: Vec<BasicMetadataTypeEnum<'ctx>> = fn_def
            .args()
            .iter()
            .map(|_| BasicMetadataTypeEnum::FloatType(f32_type))
            .collect();

        let fn_type = f32_type.fn_type(&fn_arg_types, false);
        let func_name = fn_def.id();
        let func = self.module.add_function(func_name.as_str(), fn_type, None);
        self.set_current_fn_value(func);

        let basic_block = self.context.append_basic_block(func, fn_def.id());
        self.builder.position_at_end(basic_block);

        for (arg, arg_name) in func.get_param_iter().zip(fn_def.args().iter()) {
            let alloca = self.create_entry_block_alloca(f32_type, arg_name);
            self.builder.build_store(alloca, arg);
            self.variables.insert(arg_name.to_string(), alloca);
        }

        match fn_def.body().as_ref() {
            FnDefBody::RogatoFn(expr) => {
                let body = self.compile_expr(expr)?;
                self.builder.build_return(Some(&body));
                if func.verify(true) {
                    self.fpm.run_on(&func);
                    self.clear_current_fn();
                    Ok(func)
                } else {
                    unsafe {
                        self.clear_current_fn();
                        func.delete();
                    }
                    Err(CompileError::FnDefValidationFailed(func_name.clone()))
                }
            }
            _ => self.unknown_error("Cannot compile function with NativeFn body!"),
        }
    }

    pub fn compile_fn_call(
        &mut self,
        id: &Identifier,
        args: &FnCallArgs,
    ) -> CompileResult<FloatValue<'ctx>> {
        match self.get_function(id.as_str()) {
            Some(fun) => {
                let mut compiled_args = Vec::with_capacity(args.len());

                for arg in args.iter() {
                    compiled_args.push(self.compile_expr(arg)?);
                }

                let argsv: Vec<BasicMetadataValueEnum> = compiled_args
                    .iter()
                    .by_ref()
                    .map(|&val| val.into())
                    .collect();

                match self
                    .builder
                    .build_call(fun, argsv.as_slice(), "tmp")
                    .try_as_basic_value()
                    .left()
                {
                    Some(value) => Ok(value.into_float_value()),
                    None => self.unknown_error("Invalid call produced."),
                }
            }
            None => Err(CompileError::FnNotDefined(id.clone())),
        }
    }

    pub fn compile_op_call(
        &mut self,
        id: &Identifier,
        left: &Expression,
        right: &Expression,
    ) -> CompileResult<FloatValue<'ctx>> {
        let left = self.compile_expr(left)?;
        let right = self.compile_expr(right)?;

        match id.as_str() {
            "+" => Ok(self.builder.build_float_add(left, right, "tmpadd")),
            "-" => Ok(self.builder.build_float_sub(left, right, "tmpsub")),
            "*" => Ok(self.builder.build_float_mul(left, right, "tmpmul")),
            "/" => Ok(self.builder.build_float_div(left, right, "tmpdiv")),
            _ => Err(CompileError::OpNotDefined(id.clone())),
        }
    }

    pub fn compile_module_def(&mut self, _mod_def: &ModuleDef) -> CompileResult<()> {
        todo!()
    }

    pub fn compile_type_def(&mut self, _mod_def: &TypeDef) -> CompileResult<()> {
        todo!()
    }

    pub fn compile_lit_expr(&mut self, literal: &Literal) -> CompileResult<FloatValue<'ctx>> {
        match literal {
            Literal::Number(num) => {
                let float_val = val::number_to_f64(num).unwrap();
                Ok(self.context.f32_type().const_float(float_val))
            }
            _ => self.unknown_error("Literals not yet implemented!"),
        }
    }

    pub fn compile_ast(&mut self, ast: &AST) -> CompileResult<()> {
        match ast {
            AST::RootComment(c) => Err(CompileError::IgnoredRootComment(c.to_owned())),
            AST::FnDef(fn_def) => {
                self.compile_fn_def(fn_def)?;
                Ok(())
            }
            AST::ModuleDef(mod_def) => self.compile_module_def(mod_def),
            AST::Use(_) => todo!(),
            AST::TypeDef(type_def) => self.compile_type_def(type_def),
        }
    }

    pub fn compile_expr(&mut self, expr: &Expression) -> CompileResult<FloatValue<'ctx>> {
        match expr {
            Expression::Commented(_c, e) => self.compile_expr(e),
            Expression::Lit(lit_expr) => self.compile_lit_expr(lit_expr),
            Expression::FnCall(id, args) => self.compile_fn_call(id, args),
            Expression::OpCall(id, left, right) => self.compile_op_call(id, left, right),

            Expression::Var(id) => match self.lookup_var(id) {
                Some(var) => Ok(self
                    .builder
                    .build_load(*var, id.as_str())
                    .into_float_value()),
                None => Err(CompileError::VarNotFound(id.clone())),
            },

            Expression::ConstOrTypeRef(_id) => todo!(),
            Expression::DBTypeRef(_id) => todo!(),
            Expression::PropFnRef(_id) => todo!(),
            Expression::EdgeProp(_id, _edge) => todo!(),
            Expression::IfElse(_if_else) => todo!(),
            Expression::Let(_let_expr) => todo!(),
            Expression::Lambda(_lambda) => todo!(),
            Expression::Query(_query) => todo!(),
            Expression::Symbol(_id) => todo!(),
            Expression::Quoted(_expr) => todo!(),
            Expression::QuotedAST(_ast) => todo!(),
            Expression::Unquoted(_expr) => todo!(),
            Expression::UnquotedAST(_ast) => todo!(),
            Expression::InlineFnDef(fn_def) => {
                self.compile_fn_def(fn_def)?;
                Ok(self.context.f32_type().const_zero()) // TODO: Hmmm?!
            }
        }
    }

    pub fn compile_program(&mut self, program: &Program) -> CompileResult<()> {
        for ast in program.iter() {
            self.compile_ast(ast)?;
        }
        Ok(())
    }
}
