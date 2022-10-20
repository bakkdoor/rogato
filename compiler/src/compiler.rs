use inkwell::{
    builder::Builder,
    context::Context,
    execution_engine::ExecutionEngine,
    module::Module,
    types::BasicMetadataTypeEnum,
    values::{BasicMetadataValueEnum, FloatValue, FunctionValue, PointerValue},
    OptimizationLevel,
};
use rogato_common::ast::{
    expression::Expression,
    fn_call::FnCallArgs,
    fn_def::{FnDef, FnDefBody},
    module_def::ModuleDef,
    type_expression::TypeDef,
    Identifier, Program, AST,
};
use std::collections::HashMap;

use crate::error::CompileError;

pub type CompileResult<T> = Result<T, CompileError>;

#[derive(Debug)]
pub struct Compiler<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    #[allow(dead_code)]
    execution_engine: ExecutionEngine<'ctx>,

    fn_value_opt: Option<FunctionValue<'ctx>>,
    variables: HashMap<String, PointerValue<'ctx>>,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context, module: Module<'ctx>) -> Self {
        let execution_engine = Compiler::new_execution_engine(&module);
        Self {
            context,
            module,
            builder: context.create_builder(),
            execution_engine,
            fn_value_opt: None,
            variables: HashMap::new(),
        }
    }

    pub fn new_with_module_name<S: ToString>(context: &'ctx Context, module_name: S) -> Self {
        let builder = context.create_builder();
        let module = context.create_module(module_name.to_string().as_str());
        let execution_engine = Compiler::new_execution_engine(&module);
        Self {
            context,
            module,
            builder,
            execution_engine,
            fn_value_opt: None,
            variables: HashMap::new(),
        }
    }

    pub fn context(&self) -> &'ctx Context {
        self.context
    }
    pub fn execution_engine(&self) -> &ExecutionEngine<'ctx> {
        &self.execution_engine
    }

    pub fn new_execution_engine(module: &Module<'ctx>) -> ExecutionEngine<'ctx> {
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

    fn create_entry_block_alloca<'a, S: Into<&'a str>>(&self, name: S) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();

        let entry = self.fn_value().get_first_basic_block().unwrap();

        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }

        builder.build_alloca(self.context.f64_type(), name.into())
    }

    /// Returns the `FunctionValue` representing the function being compiled.
    #[inline]
    fn fn_value(&self) -> FunctionValue<'ctx> {
        self.fn_value_opt.unwrap()
    }

    /// Sets the `FunctionValue` representing the function being compiled.
    #[inline]
    fn set_fn_value(&mut self, fn_val: FunctionValue<'ctx>) {
        self.fn_value_opt = Some(fn_val)
    }

    /// Gets a defined function given its name.
    #[inline]
    fn get_function(&self, name: &str) -> Option<FunctionValue<'ctx>> {
        self.module.get_function(name)
    }

    fn unknown_error<T, S: ToString>(&self, message: S) -> CompileResult<T> {
        Err(CompileError::Unknown(message.to_string()))
    }

    pub fn compile_fn_def(&mut self, fn_def: &FnDef) -> CompileResult<()> {
        let f32_type = self.context.f32_type();
        let fn_type = f32_type.fn_type(
            &[
                BasicMetadataTypeEnum::FloatType(f32_type),
                BasicMetadataTypeEnum::FloatType(f32_type),
                BasicMetadataTypeEnum::FloatType(f32_type),
            ],
            false,
        );
        let func_name = fn_def.id();
        let func = self.module.add_function(func_name.as_str(), fn_type, None);
        self.set_fn_value(func);

        let basic_block = self.context.append_basic_block(func, fn_def.id());
        self.builder.position_at_end(basic_block);

        let args_with_idx: HashMap<Identifier, u32> = HashMap::from_iter(
            fn_def
                .args()
                .iter()
                .cloned()
                .zip(0..fn_def.args().len().try_into().unwrap()),
        );

        for (arg, idx) in args_with_idx.iter() {
            let pointer_val = self.create_entry_block_alloca(arg.as_str());
            self.store_var(arg, pointer_val);

            self.builder.build_store(
                pointer_val,
                func.get_nth_param(*idx).unwrap().into_float_value(),
            );
        }

        match fn_def.body().as_ref() {
            FnDefBody::RogatoFn(expr) => {
                let body = self.compile_expr(expr)?;
                self.builder.build_return(Some(&body));
                Ok(())
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

    pub fn compile_ast(&mut self, ast: &AST) -> CompileResult<()> {
        match ast {
            AST::RootComment(c) => Err(CompileError::IgnoredRootComment(c.to_owned())),
            AST::FnDef(fn_def) => self.compile_fn_def(fn_def),
            AST::ModuleDef(mod_def) => self.compile_module_def(mod_def),
            AST::Use(_) => todo!(),
            AST::TypeDef(type_def) => self.compile_type_def(type_def),
        }
    }

    pub fn compile_expr(&mut self, expr: &Expression) -> CompileResult<FloatValue<'ctx>> {
        match expr {
            Expression::Commented(_c, e) => self.compile_expr(e),
            Expression::Lit(_lit_expr) => todo!(),
            Expression::FnCall(fn_ident, args) => self.compile_fn_call(fn_ident, args),
            Expression::OpCall(op_ident, left, right) => {
                self.compile_op_call(op_ident, left, right)
            }

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
