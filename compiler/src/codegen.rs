use inkwell::{
    builder::Builder,
    context::Context,
    execution_engine::ExecutionEngine,
    module::Module,
    passes::PassManager,
    types::{BasicMetadataTypeEnum, BasicType},
    values::{AnyValue, BasicMetadataValueEnum, FloatValue, FunctionValue, PointerValue},
    FloatPredicate, OptimizationLevel,
};
use rogato_common::{
    ast::{
        expression::Expression,
        fn_call::{FnCall, FnCallArgs},
        fn_def::{FnDef, FnDefBody, FnDefVariant},
        if_else::IfElse,
        literal::Literal,
        module_def::ModuleDef,
        pattern::Pattern,
        type_expression::TypeDef,
        Identifier, Program, AST,
    },
    val,
};
use std::collections::HashMap;

use crate::error::CodegenError;

pub type CodegenResult<T> = Result<T, CodegenError>;

#[inline]
fn unknown_error<S: Into<String>>(message: S) -> CodegenError {
    CodegenError::Unknown(message.into())
}

#[derive(Debug)]
pub struct Codegen<'a, 'ctx> {
    pub module: &'a Module<'ctx>,
    pub builder: &'a Builder<'ctx>,
    pub fpm: &'a PassManager<FunctionValue<'ctx>>,
    pub execution_engine: &'a ExecutionEngine<'ctx>,

    context: &'ctx Context,
    current_fn_value: Option<FunctionValue<'ctx>>,
    variables: HashMap<String, PointerValue<'ctx>>,
}

impl<'a, 'ctx> Codegen<'a, 'ctx> {
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

    pub fn new_context() -> Context {
        Context::create()
    }

    pub fn default_execution_engine(module: &'a Module<'ctx>) -> ExecutionEngine<'ctx> {
        module
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap()
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

    pub fn codegen_fn_def(&mut self, fn_def: &FnDef) -> CodegenResult<FunctionValue<'ctx>> {
        let f32_type = self.context.f32_type();

        // TODO: add support for multiple fn variants
        let FnDefVariant(args, body) = fn_def.get_variant(0).unwrap();

        let fn_arg_types: Vec<BasicMetadataTypeEnum<'ctx>> = args
            .iter()
            .map(|_| BasicMetadataTypeEnum::FloatType(f32_type))
            .collect();

        let fn_type = f32_type.fn_type(&fn_arg_types, false);
        let func_name = fn_def.id();
        let func = self.module.add_function(func_name, fn_type, None);
        self.set_current_fn_value(func);

        let basic_block = self.context.append_basic_block(func, func_name);
        self.builder.position_at_end(basic_block);

        for (arg, arg_name) in func.get_param_iter().zip(args.iter()) {
            match &**arg_name {
                Pattern::Var(arg_name) => {
                    let alloca = self.create_entry_block_alloca(f32_type, arg_name.as_str());
                    self.builder.build_store(alloca, arg);
                    self.store_var(arg_name, alloca)
                }
                _ => todo!("pattern matching not yet supported"),
            }
        }

        match body.as_ref() {
            FnDefBody::RogatoFn(expr) => {
                let body = self.codegen_expr(expr)?;
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
                    Err(CodegenError::FnDefValidationFailed(func_name.clone()))
                }
            }
            _ => Err(unknown_error("Cannot compile function with NativeFn body!")),
        }
    }

    pub fn codegen_fn_call(&mut self, fn_call: &FnCall) -> CodegenResult<FloatValue<'ctx>> {
        let id = &fn_call.id;
        let args = &fn_call.args;

        let function = self
            .get_function(id.as_str())
            .ok_or_else(|| CodegenError::FnNotDefined(id.clone()))?;

        if rogato_common::util::is_debug_enabled() {
            println!(
                "📟📟📟📟📟📟📟📟📟📟📟📟📟📟📟📟📟📟📟📟📟📟📟📟📟\n\n{}\n📟📟📟📟📟📟📟📟📟📟📟📟📟📟📟📟📟📟📟📟📟📟📟📟📟\n",
                function.print_to_string().to_string()
            );
        }

        let mut compiled_args = Vec::with_capacity(args.len());

        for arg in args.iter() {
            compiled_args.push(self.codegen_expr(arg)?);
        }

        let argsv: Vec<BasicMetadataValueEnum> = compiled_args
            .iter()
            .by_ref()
            .map(|&val| val.into())
            .collect();

        let value = self
            .builder
            .build_call(function, argsv.as_slice(), "tmp")
            .try_as_basic_value()
            .left()
            .ok_or_else(|| unknown_error("Invalid call produced."))?;

        Ok(value.into_float_value())
    }

    pub fn codegen_op_call(
        &mut self,
        id: &Identifier,
        left: &Expression,
        right: &Expression,
    ) -> CodegenResult<FloatValue<'ctx>> {
        let left = self.codegen_expr(left)?;
        let right = self.codegen_expr(right)?;

        match id.as_str() {
            "+" => Ok(self.builder.build_float_add(left, right, "tmp_add")),
            "-" => Ok(self.builder.build_float_sub(left, right, "tmp_sub")),
            "*" => Ok(self.builder.build_float_mul(left, right, "tmp_mul")),
            "/" => Ok(self.builder.build_float_div(left, right, "tmp_div")),
            "%" => Ok(self.builder.build_float_rem(left, right, "tmp_rem")),
            ">" => {
                let int_val =
                    self.builder
                        .build_float_compare(FloatPredicate::OGT, left, right, "gt");
                Ok(self.builder.build_unsigned_int_to_float(
                    int_val,
                    left.get_type(),
                    "bool_to_float",
                ))
            }
            "<" => {
                let int_val =
                    self.builder
                        .build_float_compare(FloatPredicate::OLT, left, right, "lt");
                Ok(self.builder.build_unsigned_int_to_float(
                    int_val,
                    left.get_type(),
                    "bool_to_float",
                ))
            }
            ">=" => {
                let int_val =
                    self.builder
                        .build_float_compare(FloatPredicate::OGE, left, right, "ge");
                Ok(self.builder.build_unsigned_int_to_float(
                    int_val,
                    left.get_type(),
                    "bool_to_float",
                ))
            }
            "<=" => {
                let int_val =
                    self.builder
                        .build_float_compare(FloatPredicate::OLE, left, right, "le");
                Ok(self.builder.build_unsigned_int_to_float(
                    int_val,
                    left.get_type(),
                    "bool_to_float",
                ))
            }
            _ => Err(CodegenError::OpNotDefined(id.clone())),
        }
    }

    pub fn codegen_module_def(&mut self, _mod_def: &ModuleDef) -> CodegenResult<()> {
        todo!()
    }

    pub fn codegen_type_def(&mut self, _mod_def: &TypeDef) -> CodegenResult<()> {
        todo!()
    }

    pub fn codegen_lit_expr(&mut self, literal: &Literal) -> CodegenResult<FloatValue<'ctx>> {
        match literal {
            Literal::Number(num) => {
                let float_val = val::number_to_f64(num).unwrap();
                Ok(self.context.f32_type().const_float(float_val))
            }
            _ => Err(unknown_error("Literals not yet implemented!")),
        }
    }

    pub fn codegen_ast(&mut self, ast: &AST) -> CodegenResult<()> {
        match ast {
            AST::RootComment(c) => Err(CodegenError::IgnoredRootComment(c.to_owned())),
            AST::FnDef(fn_def) => {
                self.codegen_fn_def(&fn_def.borrow())?;
                Ok(())
            }
            AST::ModuleDef(mod_def) => self.codegen_module_def(mod_def),
            AST::Use(_id, _imports) => todo!(),
            AST::TypeDef(type_def) => self.codegen_type_def(type_def),
        }
    }

    pub fn codegen_expr(&mut self, expr: &Expression) -> CodegenResult<FloatValue<'ctx>> {
        match expr {
            Expression::Commented(_c, e) => self.codegen_expr(e),
            Expression::Lit(lit_expr) => self.codegen_lit_expr(lit_expr),
            Expression::FnCall(fn_call) => self.codegen_fn_call(fn_call),
            Expression::OpCall(id, left, right) => self.codegen_op_call(id, left, right),

            Expression::Var(id) => match self.lookup_var(id) {
                Some(var) => Ok(self
                    .builder
                    .build_load(*var, id.as_str())
                    .into_float_value()),
                None => {
                    self.codegen_fn_call(&FnCall::new(id.into(), FnCallArgs::empty()))
                    // Err(CompileError::VarNotFound(id.clone()))
                }
            },

            Expression::ConstOrTypeRef(_id) => todo!(),
            Expression::DBTypeRef(_id) => todo!(),
            Expression::PropFnRef(_id) => todo!(),
            Expression::EdgeProp(_id, _edge) => todo!(),
            Expression::IfElse(if_else) => self.codegen_if_else(if_else),
            Expression::Let(_let_expr) => todo!(),
            Expression::Lambda(_lambda) => todo!(),
            Expression::Query(_query) => todo!(),
            Expression::Symbol(_id) => todo!(),
            Expression::Quoted(_expr) => todo!(),
            Expression::QuotedAST(_ast) => todo!(),
            Expression::Unquoted(_expr) => todo!(),
            Expression::UnquotedAST(_ast) => todo!(),
            Expression::InlineFnDef(fn_def) => {
                self.codegen_fn_def(&fn_def.borrow())?;
                Ok(self.context.f32_type().const_zero()) // TODO: Hmmm?!
            }
        }
    }

    fn codegen_if_else(&mut self, if_else: &IfElse) -> CodegenResult<FloatValue<'ctx>> {
        let cond = self.codegen_expr(&if_else.condition)?;
        let zero = self.context.f32_type().const_zero();
        let cmp =
            self.builder
                .build_float_compare(inkwell::FloatPredicate::ONE, cond, zero, "ifcond");

        let then_block = self
            .context
            .append_basic_block(self.current_fn_value(), "then");
        let else_block = self
            .context
            .append_basic_block(self.current_fn_value(), "else");
        let merge_block = self
            .context
            .append_basic_block(self.current_fn_value(), "merge");

        self.builder
            .build_conditional_branch(cmp, then_block, else_block);

        self.builder.position_at_end(then_block);
        let then_value = self.codegen_expr(&if_else.then_expr)?;
        self.builder.build_unconditional_branch(merge_block);

        self.builder.position_at_end(else_block);
        let else_value = self.codegen_expr(&if_else.else_expr)?;
        self.builder.build_unconditional_branch(merge_block);

        self.builder.position_at_end(merge_block);
        let phi = self.builder.build_phi(self.context.f32_type(), "if-phi");
        phi.add_incoming(&[(&then_value, then_block), (&else_value, else_block)]);

        Ok(phi.as_basic_value().into_float_value())
    }

    pub fn codegen_program(&mut self, program: &Program) -> CodegenResult<()> {
        for ast in program.iter() {
            self.codegen_ast(ast)?;
        }
        Ok(())
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

    pub fn new_module(&self, name: &str) -> Module<'ctx> {
        self.context.create_module(name)
    }

    fn lookup_var<S: ToString>(&self, name: S) -> Option<&PointerValue<'ctx>> {
        self.variables.get(&name.to_string())
    }

    #[allow(dead_code)]
    fn store_var<S: ToString>(&mut self, name: S, pointer_val: PointerValue<'ctx>) {
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
}
