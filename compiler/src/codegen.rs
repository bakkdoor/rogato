use inkwell::{
    builder::Builder,
    context::Context,
    execution_engine::ExecutionEngine,
    module::Module,
    passes::PassBuilderOptions,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
    types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, IntType, PointerType},
    values::{
        AnyValue, BasicMetadataValueEnum, BasicValueEnum, FloatValue, FunctionValue, IntValue,
        PointerValue,
    },
    AddressSpace, FloatPredicate, IntPredicate, OptimizationLevel,
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
        type_expression::{TypeDef, TypeExpression},
        Identifier, Program, VarIdentifier, AST,
    },
    val,
};
use std::{cell::RefCell, collections::HashMap, ops::Deref, rc::Rc};

use crate::error::CodegenError;

#[derive(Debug, Clone)]
pub enum CompiledValue<'ctx> {
    Float(FloatValue<'ctx>),
    Int32(IntValue<'ctx>),
    Int64(IntValue<'ctx>),
    String(PointerValue<'ctx>),
    Bool(IntValue<'ctx>),
}

impl<'ctx> CompiledValue<'ctx> {
    pub fn into_basic_value(self) -> BasicValueEnum<'ctx> {
        match self {
            CompiledValue::Float(v) => v.into(),
            CompiledValue::Int32(v) => v.into(),
            CompiledValue::Int64(v) => v.into(),
            CompiledValue::String(v) => v.into(),
            CompiledValue::Bool(v) => v.into(),
        }
    }

    pub fn as_basic_value(&self) -> BasicValueEnum<'ctx> {
        match self {
            CompiledValue::Float(v) => (*v).into(),
            CompiledValue::Int32(v) => (*v).into(),
            CompiledValue::Int64(v) => (*v).into(),
            CompiledValue::String(v) => (*v).into(),
            CompiledValue::Bool(v) => (*v).into(),
        }
    }

    pub fn get_type(&self) -> CompiledType {
        match self {
            CompiledValue::Float(_) => CompiledType::Float,
            CompiledValue::Int32(_) => CompiledType::Int32,
            CompiledValue::Int64(_) => CompiledType::Int64,
            CompiledValue::String(_) => CompiledType::String,
            CompiledValue::Bool(_) => CompiledType::Bool,
        }
    }
}

pub type CodegenResult<T> = Result<T, CodegenError>;

#[inline]
fn unknown_error<S: Into<String>>(message: S) -> CodegenError {
    CodegenError::Unknown(message.into())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompiledType {
    Float,
    Int32,
    Int64,
    String,
    Bool,
}

impl<'ctx> CompiledType {
    #[allow(dead_code)]
    pub fn from_type_expression(te: &rogato_common::ast::type_expression::TypeExpression) -> Self {
        let te_str = format!("{:?}", te);
        if te_str.contains("Int32") {
            CompiledType::Int32
        } else if te_str.contains("Int64") {
            CompiledType::Int64
        } else if te_str.contains("String") || te_str.contains("Symbol") {
            CompiledType::String
        } else if te_str.contains("Bool") || te_str.contains("True") {
            CompiledType::Bool
        } else {
            CompiledType::Float
        }
    }

    pub fn as_basic_type_enum(&self, ctx: &'ctx Context) -> BasicTypeEnum<'ctx> {
        match self {
            CompiledType::Float => ctx.f32_type().into(),
            CompiledType::Int32 => ctx.i32_type().into(),
            CompiledType::Int64 => ctx.i64_type().into(),
            CompiledType::String => ctx.i8_type().ptr_type(AddressSpace::default()).into(),
            CompiledType::Bool => ctx.bool_type().into(),
        }
    }

    pub fn as_metadata_type_enum(&self, ctx: &'ctx Context) -> BasicMetadataTypeEnum<'ctx> {
        match self {
            CompiledType::Float => BasicMetadataTypeEnum::FloatType(ctx.f32_type()),
            CompiledType::Int32 => BasicMetadataTypeEnum::IntType(ctx.i32_type()),
            CompiledType::Int64 => BasicMetadataTypeEnum::IntType(ctx.i64_type()),
            CompiledType::String => {
                BasicMetadataTypeEnum::PointerType(ctx.i8_type().ptr_type(AddressSpace::default()))
            }
            CompiledType::Bool => BasicMetadataTypeEnum::IntType(ctx.bool_type()),
        }
    }
}

#[derive(Debug)]
pub struct Codegen<'a, 'ctx> {
    pub module: &'a Module<'ctx>,
    pub builder: &'a Builder<'ctx>,
    pub target_machine: &'a TargetMachine,
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
        target_machine: &'a TargetMachine,
        execution_engine: &'a ExecutionEngine<'ctx>,
    ) -> Self {
        Self {
            context,
            module,
            builder,
            target_machine,
            execution_engine,
            current_fn_value: None,
            variables: HashMap::new(),
        }
    }

    pub fn new_context() -> Context {
        Context::create()
    }

    #[inline]
    pub fn i32_type(&self) -> IntType<'ctx> {
        self.context.i32_type()
    }

    #[inline]
    pub fn i64_type(&self) -> IntType<'ctx> {
        self.context.i64_type()
    }

    #[inline]
    pub fn i8_type(&self) -> IntType<'ctx> {
        self.context.i8_type()
    }

    #[inline]
    pub fn string_type(&self) -> PointerType<'ctx> {
        self.i8_type().ptr_type(AddressSpace::default())
    }

    #[inline]
    pub fn bool_type(&self) -> IntType<'ctx> {
        self.context.bool_type()
    }

    pub fn default_execution_engine(module: &'a Module<'ctx>) -> ExecutionEngine<'ctx> {
        module
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap()
    }

    pub fn default_target_machine(_module: &Module<'ctx>) -> TargetMachine {
        Target::initialize_native(&InitializationConfig::default()).ok();

        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple).unwrap();
        target
            .create_target_machine(
                &target_triple,
                "",
                "",
                OptimizationLevel::None,
                RelocMode::Default,
                CodeModel::Default,
            )
            .unwrap()
    }

    pub fn run_function_passes(&self) {
        let passes = "default<O0>";
        let options = PassBuilderOptions::create();
        self.module
            .run_passes(passes, self.target_machine, options)
            .ok();
    }

    pub fn declare_fn_signature(&mut self, fn_def: &FnDef) -> CodegenResult<FunctionValue<'ctx>> {
        let func_name = fn_def.id();

        if let Some(existing) = self.module.get_function(func_name.as_str()) {
            return Ok(existing);
        }

        let first_variant = match fn_def.variants_iter().next() {
            Some(v) => v,
            None => return Err(unknown_error("Function has no variants")),
        };

        let args = &first_variant.0;
        let body = &first_variant.1;

        let return_type = match first_variant.return_type() {
            Some(rexpr) => CompiledType::from_type_expression(rexpr),
            None => match body.as_ref() {
                FnDefBody::RogatoFn(expr) => self.infer_fn_arg_types(expr, args.len()),
                _ => CompiledType::Float,
            },
        };

        let return_llvm_type = return_type.as_basic_type_enum(self.context);

        let fn_arg_types: Vec<BasicMetadataTypeEnum<'ctx>> = args
            .iter()
            .map(|_| BasicMetadataTypeEnum::FloatType(self.context.f32_type()))
            .collect();

        let fn_type = return_llvm_type.fn_type(&fn_arg_types, false);

        let func = self.module.add_function(func_name, fn_type, None);
        Ok(func)
    }

    fn infer_fn_arg_types(&self, expr: &Expression, arg_count: usize) -> CompiledType {
        use rogato_type_checker::TypeInferrer;

        let mut inferrer = TypeInferrer::new();

        for i in 0..arg_count {
            let arg_id: VarIdentifier = format!("_arg_{}", i).as_str().into();
            inferrer
                .env_mut()
                .insert_variable(arg_id, Rc::new(TypeExpression::Unknown));
        }

        match inferrer.infer_expression(expr) {
            rogato_type_checker::InferredType::Known(type_expr) => match type_expr.deref() {
                TypeExpression::FunctionType(_lambda_args, return_type) => {
                    CompiledType::from_type_expression(return_type)
                }
                _ => CompiledType::Float,
            },
            rogato_type_checker::InferredType::Unknown => CompiledType::Float,
        }
    }

    pub fn codegen_fn_def(&mut self, fn_def: &FnDef) -> CodegenResult<FunctionValue<'ctx>> {
        let f32_type = self.context.f32_type();
        let func_name = fn_def.id();

        if self.module.get_function(func_name.as_str()).is_some() {
            return Err(CodegenError::FnNotDefined(func_name.clone()));
        }

        let variants: Vec<_> = fn_def.variants_iter().collect();

        if variants.len() > 1 {
            return self.codegen_multi_variant_fn(fn_def);
        }

        let FnDefVariant(args, body, return_type) = fn_def.get_variant(0).unwrap();

        let return_type = match return_type {
            Some(rexpr) => CompiledType::from_type_expression(rexpr),
            None => match body.as_ref() {
                FnDefBody::RogatoFn(expr) => self.infer_expr_type_with_checker(expr),
                _ => CompiledType::Float,
            },
        };

        let return_llvm_type = return_type.as_basic_type_enum(self.context);

        let fn_arg_types: Vec<BasicMetadataTypeEnum<'ctx>> = args
            .iter()
            .map(|_| BasicMetadataTypeEnum::FloatType(f32_type))
            .collect();

        let fn_type = return_llvm_type.fn_type(&fn_arg_types, false);
        let func = self.module.add_function(func_name, fn_type, None);

        self.codegen_fn_body(fn_def, func)
    }

    fn codegen_fn_body(
        &mut self,
        fn_def: &FnDef,
        func: FunctionValue<'ctx>,
    ) -> CodegenResult<FunctionValue<'ctx>> {
        let f32_type = self.context.f32_type();
        let FnDefVariant(args, body, return_type) = fn_def.get_variant(0).unwrap();

        self.set_current_fn_value(func);

        let basic_block = self.context.append_basic_block(func, fn_def.id());
        self.builder.position_at_end(basic_block);

        for (arg, arg_name) in func.get_param_iter().zip(args.iter()) {
            match &**arg_name {
                Pattern::Var(arg_name) => {
                    let alloca = self.create_entry_block_alloca(f32_type, arg_name.as_str());
                    self.builder.build_store(alloca, arg)?;
                    self.store_var(arg_name, alloca)
                }
                _ => todo!("pattern matching not yet supported"),
            }
        }

        match body.as_ref() {
            FnDefBody::RogatoFn(expr) => {
                let compiled_val = self.codegen_expr(expr)?;
                let ret_val = compiled_val.into_basic_value();
                self.builder.build_return(Some(&ret_val))?;

                if func.verify(true) {
                    self.run_function_passes();
                    self.clear_current_fn();
                    Ok(func)
                } else {
                    unsafe {
                        self.clear_current_fn();
                        func.delete();
                    }
                    Err(CodegenError::FnDefValidationFailed(fn_def.id().clone()))
                }
            }
            _ => Err(unknown_error("Cannot compile function with NativeFn body!")),
        }
    }

    fn codegen_multi_variant_fn(&mut self, fn_def: &FnDef) -> CodegenResult<FunctionValue<'ctx>> {
        let variants: Vec<_> = fn_def.variants_iter().collect();

        if variants.is_empty() {
            return Err(CodegenError::FnDefValidationFailed(fn_def.id().clone()));
        }

        let first_variant = &variants[0];
        let arg_count = first_variant.0.len();

        for variant in variants.iter() {
            if variant.0.len() != arg_count {
                return Err(CodegenError::FnDefValidationFailed(fn_def.id().clone()));
            }
        }

        let has_catch_all = variants
            .last()
            .map(|v| {
                v.0.iter()
                    .any(|p| matches!(p.deref(), Pattern::Var(_) | Pattern::Any))
            })
            .unwrap_or(false);

        if !has_catch_all {
            return Err(CodegenError::FnPatternUncovered(fn_def.id().clone()));
        }

        let func_name = fn_def.id();

        let return_type = match &first_variant.2 {
            Some(rexpr) => CompiledType::from_type_expression(rexpr),
            None => match first_variant.1.deref() {
                FnDefBody::RogatoFn(expr) => self.infer_expr_type_with_checker(expr),
                _ => CompiledType::Float,
            },
        };

        let return_llvm_type = return_type.as_basic_type_enum(self.context);

        let f32_type = self.context.f32_type();
        let fn_arg_types: Vec<BasicMetadataTypeEnum<'ctx>> = (0..arg_count)
            .map(|_| BasicMetadataTypeEnum::FloatType(f32_type))
            .collect();

        let fn_type = return_llvm_type.fn_type(&fn_arg_types, false);
        let func = self.module.add_function(func_name.as_str(), fn_type, None);

        self.set_current_fn_value(func);

        let entry_block = self.context.append_basic_block(func, "entry");
        self.builder.position_at_end(entry_block);

        let params: Vec<_> = func.get_param_iter().collect();

        self.codegen_variant_body(fn_def, 0, &params)?;

        if func.verify(true) {
            self.run_function_passes();
            self.clear_current_fn();
            Ok(func)
        } else {
            unsafe {
                self.clear_current_fn();
                func.delete();
            }
            Err(CodegenError::FnDefValidationFailed(fn_def.id().clone()))
        }
    }

    fn codegen_variant_body(
        &mut self,
        fn_def: &FnDef,
        variant_index: usize,
        params: &[inkwell::values::BasicValueEnum<'ctx>],
    ) -> CodegenResult<()> {
        let f32_type = self.context.f32_type();
        let variants: Vec<_> = fn_def.variants_iter().collect();

        if variant_index >= variants.len() {
            return Ok(());
        }

        let current_variant = &variants[variant_index];
        let FnDefVariant(args, body, _return_type) = current_variant;

        if variant_index < variants.len() - 1 {
            let next_variant = &variants[variant_index + 1];

            let has_catch_all_at_position: Vec<bool> = args
                .iter()
                .enumerate()
                .map(|(i, p)| match p.deref() {
                    Pattern::Var(_) | Pattern::Any => true,
                    _ => false,
                })
                .collect();

            let first_catch_all = args
                .iter()
                .position(|p| matches!(p.deref(), Pattern::Var(_) | Pattern::Any));

            if let Some(catch_all_idx) = first_catch_all {
                for (i, arg_name) in args.iter().enumerate() {
                    match &**arg_name {
                        Pattern::Var(var_id) => {
                            let alloca = self.create_entry_block_alloca(f32_type, var_id.as_str());
                            self.builder.build_store(alloca, params[i])?;
                            self.store_var(var_id.as_str(), alloca);
                        }
                        _ => {}
                    }
                }

                let variant_body_block = self.context.append_basic_block(
                    self.current_fn_value(),
                    &format!("variant_{}_body", variant_index),
                );

                let merge_block = self.context.append_basic_block(
                    self.current_fn_value(),
                    &format!("variant_{}_merge", variant_index),
                );

                self.builder
                    .build_unconditional_branch(variant_body_block)?;

                self.builder.position_at_end(variant_body_block);

                match body.deref() {
                    FnDefBody::RogatoFn(expr) => {
                        let compiled_val = self.codegen_expr(expr)?;
                        let ret_val = compiled_val.into_basic_value();

                        if variant_index < variants.len() - 1 {
                            let next_body_block = self.context.append_basic_block(
                                self.current_fn_value(),
                                &format!("variant_{}_body", variant_index + 1),
                            );
                            self.builder.build_return(Some(&ret_val))?;

                            return self.codegen_variant_body_with_block(
                                fn_def,
                                variant_index + 1,
                                params,
                                Some(next_body_block),
                            );
                        }
                    }
                    _ => return Err(unknown_error("Cannot compile function with NativeFn body!")),
                }
            } else {
                let mut conditions: Vec<(usize, IntValue<'ctx>)> = Vec::new();

                for (i, arg_pattern) in args.iter().enumerate() {
                    match &**arg_pattern {
                        Pattern::Number(num) => {
                            let num_val = val::number_to_f64(num).unwrap_or(0.0);
                            let const_val = f32_type.const_float(num_val);

                            let cmp = self.builder.build_float_compare(
                                FloatPredicate::OEQ,
                                params[i].into_float_value(),
                                const_val,
                                &format!("cmp_arg_{}_variant_{}", i, variant_index),
                            )?;

                            conditions.push((i, cmp));
                        }
                        Pattern::Var(_) | Pattern::Any => {
                            break;
                        }
                        _ => {}
                    }
                }

                if !conditions.is_empty() {
                    let last_condition = conditions.last().unwrap();

                    let next_variant_block = self.context.append_basic_block(
                        self.current_fn_value(),
                        &format!("test_variant_{}", variant_index + 1),
                    );

                    let current_variant_block = self.context.append_basic_block(
                        self.current_fn_value(),
                        &format!("variant_{}_body", variant_index),
                    );

                    self.builder.build_conditional_branch(
                        last_condition.1,
                        current_variant_block,
                        next_variant_block,
                    )?;

                    self.builder.position_at_end(current_variant_block);

                    for (i, arg_name) in args.iter().enumerate() {
                        match &**arg_name {
                            Pattern::Var(var_id) => {
                                let alloca =
                                    self.create_entry_block_alloca(f32_type, var_id.as_str());
                                self.builder.build_store(alloca, params[i])?;
                                self.store_var(var_id.as_str(), alloca);
                            }
                            _ => {}
                        }
                    }

                    match body.deref() {
                        FnDefBody::RogatoFn(expr) => {
                            let compiled_val = self.codegen_expr(expr)?;
                            let ret_val = compiled_val.into_basic_value();

                            if variant_index < variants.len() - 1 {
                                let return_block = self.context.append_basic_block(
                                    self.current_fn_value(),
                                    &format!("variant_{}_return", variant_index),
                                );

                                self.builder.build_unconditional_branch(return_block)?;
                                self.builder.position_at_end(return_block);

                                let next_body_block = self.context.append_basic_block(
                                    self.current_fn_value(),
                                    &format!("variant_{}_body", variant_index + 1),
                                );

                                self.builder.build_return(Some(&ret_val))?;

                                return self.codegen_variant_body_with_block(
                                    fn_def,
                                    variant_index + 1,
                                    params,
                                    Some(next_body_block),
                                );
                            }
                        }
                        _ => {
                            return Err(unknown_error(
                                "Cannot compile function with NativeFn body!",
                            ))
                        }
                    }
                } else {
                    let variant_body_block = self.context.append_basic_block(
                        self.current_fn_value(),
                        &format!("variant_{}_body", variant_index),
                    );

                    self.builder
                        .build_unconditional_branch(variant_body_block)?;

                    self.builder.position_at_end(variant_body_block);

                    for (i, arg_name) in args.iter().enumerate() {
                        match &**arg_name {
                            Pattern::Var(var_id) => {
                                let alloca =
                                    self.create_entry_block_alloca(f32_type, var_id.as_str());
                                self.builder.build_store(alloca, params[i])?;
                                self.store_var(var_id.as_str(), alloca);
                            }
                            _ => {}
                        }
                    }

                    match body.deref() {
                        FnDefBody::RogatoFn(expr) => {
                            let compiled_val = self.codegen_expr(expr)?;
                            let ret_val = compiled_val.into_basic_value();

                            if variant_index < variants.len() - 1 {
                                return Ok(());
                            }

                            self.builder.build_return(Some(&ret_val))?;
                        }
                        _ => {
                            return Err(unknown_error(
                                "Cannot compile function with NativeFn body!",
                            ))
                        }
                    }
                }
            }
        } else {
            let variant_body_block = self.context.append_basic_block(
                self.current_fn_value(),
                &format!("variant_{}_body", variant_index),
            );

            self.builder
                .build_unconditional_branch(variant_body_block)?;

            self.builder.position_at_end(variant_body_block);

            for (i, arg_name) in args.iter().enumerate() {
                match &**arg_name {
                    Pattern::Var(var_id) => {
                        let alloca = self.create_entry_block_alloca(f32_type, var_id.as_str());
                        self.builder.build_store(alloca, params[i])?;
                        self.store_var(var_id.as_str(), alloca);
                    }
                    _ => {}
                }
            }

            match body.deref() {
                FnDefBody::RogatoFn(expr) => {
                    let compiled_val = self.codegen_expr(expr)?;
                    let ret_val = compiled_val.into_basic_value();
                    self.builder.build_return(Some(&ret_val))?;
                }
                _ => return Err(unknown_error("Cannot compile function with NativeFn body!")),
            }
        }

        Ok(())
    }

    fn codegen_variant_body_with_block(
        &mut self,
        fn_def: &FnDef,
        variant_index: usize,
        params: &[inkwell::values::BasicValueEnum<'ctx>],
        entry_block: Option<inkwell::basic_block::BasicBlock<'ctx>>,
    ) -> CodegenResult<()> {
        if let Some(block) = entry_block {
            self.builder.position_at_end(block);
        }

        if variant_index >= fn_def.variants_iter().count() {
            return Ok(());
        }

        let f32_type = self.context.f32_type();

        if variant_index >= 1 {
            let variants: Vec<_> = fn_def.variants_iter().collect();

            for (i, arg_name) in variants[variant_index].0.iter().enumerate() {
                match &**arg_name {
                    Pattern::Var(var_id) => {
                        if self.lookup_var(var_id.as_str()).is_none() {
                            let alloca = self.create_entry_block_alloca(f32_type, var_id.as_str());
                            self.builder.build_store(alloca, params[i])?;
                            self.store_var(var_id.as_str(), alloca);
                        }
                    }
                    _ => {}
                }
            }
        }

        self.codegen_variant_body(fn_def, variant_index, params)
    }

    fn infer_expr_type_with_checker(&self, expr: &Expression) -> CompiledType {
        use rogato_type_checker::TypeInferrer;

        let inferrer = TypeInferrer::new();
        match inferrer.infer_expression(expr) {
            rogato_type_checker::InferredType::Known(type_expr) => {
                CompiledType::from_type_expression(&type_expr)
            }
            rogato_type_checker::InferredType::Unknown => CompiledType::Float,
        }
    }

    pub fn codegen_program(&mut self, program: &Program) -> CodegenResult<()> {
        let mut fn_defs_by_name: HashMap<Identifier, Vec<Rc<RefCell<FnDef>>>> = HashMap::new();

        for ast in program.iter() {
            if let AST::FnDef(fn_def) = ast.as_ref() {
                fn_defs_by_name
                    .entry(fn_def.borrow().id().clone())
                    .or_insert_with(Vec::new)
                    .push(Rc::clone(fn_def));
            }
        }

        let combined_fn_defs: Vec<Rc<RefCell<FnDef>>> = fn_defs_by_name
            .into_values()
            .map(|def_vec| {
                if def_vec.len() == 1 {
                    Rc::clone(&def_vec[0])
                } else {
                    let first_fn_def = &def_vec[0].borrow();

                    let variants: Vec<_> = def_vec
                        .iter()
                        .flat_map(|f| f.borrow().variants_iter().cloned())
                        .collect();

                    let combined_id = first_fn_def.id().clone();
                    FnDef::new_with_variants(combined_id, variants)
                }
            })
            .collect();

        for fn_def in combined_fn_defs.iter() {
            self.declare_fn_signature(&fn_def.borrow())?;
        }

        for fn_def in combined_fn_defs.iter() {
            self.codegen_fn_def(&fn_def.borrow())?;
        }

        Ok(())
    }

    pub fn codegen_fn_call(&mut self, fn_call: &FnCall) -> CodegenResult<CompiledValue<'ctx>> {
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
            let compiled_val = self.codegen_expr(arg)?;
            compiled_args.push(compiled_val.into_basic_value());
        }

        let argsv: Vec<BasicMetadataValueEnum> = compiled_args
            .iter()
            .by_ref()
            .map(|val| (*val).into())
            .collect();

        let call_site = self.builder.build_call(function, argsv.as_slice(), "tmp")?;

        let value = call_site
            .try_as_basic_value()
            .basic()
            .ok_or_else(|| unknown_error("Invalid call produced."))?;

        Ok(CompiledValue::Float(value.into_float_value()))
    }

    pub fn codegen_op_call(
        &mut self,
        id: &Identifier,
        left_expr: &Expression,
        right_expr: &Expression,
    ) -> CodegenResult<CompiledValue<'ctx>> {
        let left = self.codegen_expr(left_expr)?;
        let right = self.codegen_expr(right_expr)?;

        match id.as_str() {
            "+" => match (&left, &right) {
                (CompiledValue::Float(l), CompiledValue::Float(r)) => Ok(CompiledValue::Float(
                    self.builder
                        .build_float_add(*l, *r, "tmp_add")
                        .map_err(|e| unknown_error(format!("{:?}", e)))?,
                )),
                (CompiledValue::Int32(l), CompiledValue::Int32(r)) => Ok(CompiledValue::Int32(
                    self.builder
                        .build_int_add(*l, *r, "tmp_add")
                        .map_err(|e| unknown_error(format!("{:?}", e)))?,
                )),
                (CompiledValue::Int64(l), CompiledValue::Int64(r)) => Ok(CompiledValue::Int64(
                    self.builder
                        .build_int_add(*l, *r, "tmp_add")
                        .map_err(|e| unknown_error(format!("{:?}", e)))?,
                )),
                _ => Err(unknown_error("Type mismatch for + operation")),
            },
            "-" => match (&left, &right) {
                (CompiledValue::Float(l), CompiledValue::Float(r)) => Ok(CompiledValue::Float(
                    self.builder
                        .build_float_sub(*l, *r, "tmp_sub")
                        .map_err(|e| unknown_error(format!("{:?}", e)))?,
                )),
                (CompiledValue::Int32(l), CompiledValue::Int32(r)) => Ok(CompiledValue::Int32(
                    self.builder
                        .build_int_sub(*l, *r, "tmp_sub")
                        .map_err(|e| unknown_error(format!("{:?}", e)))?,
                )),
                (CompiledValue::Int64(l), CompiledValue::Int64(r)) => Ok(CompiledValue::Int64(
                    self.builder
                        .build_int_sub(*l, *r, "tmp_sub")
                        .map_err(|e| unknown_error(format!("{:?}", e)))?,
                )),
                _ => Err(unknown_error("Type mismatch for - operation")),
            },
            "*" => match (&left, &right) {
                (CompiledValue::Float(l), CompiledValue::Float(r)) => Ok(CompiledValue::Float(
                    self.builder
                        .build_float_mul(*l, *r, "tmp_mul")
                        .map_err(|e| unknown_error(format!("{:?}", e)))?,
                )),
                (CompiledValue::Int32(l), CompiledValue::Int32(r)) => Ok(CompiledValue::Int32(
                    self.builder
                        .build_int_mul(*l, *r, "tmp_mul")
                        .map_err(|e| unknown_error(format!("{:?}", e)))?,
                )),
                (CompiledValue::Int64(l), CompiledValue::Int64(r)) => Ok(CompiledValue::Int64(
                    self.builder
                        .build_int_mul(*l, *r, "tmp_mul")
                        .map_err(|e| unknown_error(format!("{:?}", e)))?,
                )),
                _ => Err(unknown_error("Type mismatch for * operation")),
            },
            "/" => match (&left, &right) {
                (CompiledValue::Float(l), CompiledValue::Float(r)) => Ok(CompiledValue::Float(
                    self.builder
                        .build_float_div(*l, *r, "tmp_div")
                        .map_err(|e| unknown_error(format!("{:?}", e)))?,
                )),
                (CompiledValue::Int32(l), CompiledValue::Int32(r)) => Ok(CompiledValue::Int32(
                    self.builder
                        .build_int_signed_div(*l, *r, "tmp_div")
                        .map_err(|e| unknown_error(format!("{:?}", e)))?,
                )),
                (CompiledValue::Int64(l), CompiledValue::Int64(r)) => Ok(CompiledValue::Int64(
                    self.builder
                        .build_int_signed_div(*l, *r, "tmp_div")
                        .map_err(|e| unknown_error(format!("{:?}", e)))?,
                )),
                _ => Err(unknown_error("Type mismatch for / operation")),
            },
            "%" => match (&left, &right) {
                (CompiledValue::Float(l), CompiledValue::Float(r)) => Ok(CompiledValue::Float(
                    self.builder
                        .build_float_rem(*l, *r, "tmp_rem")
                        .map_err(|e| unknown_error(format!("{:?}", e)))?,
                )),
                (CompiledValue::Int32(l), CompiledValue::Int32(r)) => Ok(CompiledValue::Int32(
                    self.builder
                        .build_int_signed_rem(*l, *r, "tmp_rem")
                        .map_err(|e| unknown_error(format!("{:?}", e)))?,
                )),
                (CompiledValue::Int64(l), CompiledValue::Int64(r)) => Ok(CompiledValue::Int64(
                    self.builder
                        .build_int_signed_rem(*l, *r, "tmp_rem")
                        .map_err(|e| unknown_error(format!("{:?}", e)))?,
                )),
                _ => Err(unknown_error("Type mismatch for % operation")),
            },
            ">" | "<" | ">=" | "<=" => {
                let op = match id.as_str() {
                    ">" => FloatPredicate::OGT,
                    "<" => FloatPredicate::OLT,
                    ">=" => FloatPredicate::OGE,
                    "<=" => FloatPredicate::OLE,
                    _ => return Err(unknown_error("Invalid operator")),
                };
                match (&left, &right) {
                    (CompiledValue::Float(l), CompiledValue::Float(r)) => {
                        let cmp = self.builder.build_float_compare(op, *l, *r, "cmp")?;
                        Ok(CompiledValue::Bool(cmp))
                    }
                    (CompiledValue::Int32(l), CompiledValue::Int32(r)) => {
                        let pred = match id.as_str() {
                            ">" => IntPredicate::SGT,
                            "<" => IntPredicate::SLT,
                            ">=" => IntPredicate::SGE,
                            "<=" => IntPredicate::SLE,
                            _ => return Err(unknown_error("Invalid operator")),
                        };
                        let cmp = self.builder.build_int_compare(pred, *l, *r, "cmp")?;
                        Ok(CompiledValue::Bool(cmp))
                    }
                    (CompiledValue::Int64(l), CompiledValue::Int64(r)) => {
                        let pred = match id.as_str() {
                            ">" => IntPredicate::SGT,
                            "<" => IntPredicate::SLT,
                            ">=" => IntPredicate::SGE,
                            "<=" => IntPredicate::SLE,
                            _ => return Err(unknown_error("Invalid operator")),
                        };
                        let cmp = self.builder.build_int_compare(pred, *l, *r, "cmp")?;
                        Ok(CompiledValue::Bool(cmp))
                    }
                    _ => Err(unknown_error("Type mismatch for comparison operation")),
                }
            }
            "==" | "!=" => {
                let op = if id.as_str() == "==" {
                    FloatPredicate::OEQ
                } else {
                    FloatPredicate::ONE
                };
                match (&left, &right) {
                    (CompiledValue::Float(l), CompiledValue::Float(r)) => {
                        let cmp = self.builder.build_float_compare(op, *l, *r, "cmp")?;
                        Ok(CompiledValue::Bool(cmp))
                    }
                    (CompiledValue::Int32(l), CompiledValue::Int32(r)) => {
                        let pred = if id.as_str() == "==" {
                            IntPredicate::EQ
                        } else {
                            IntPredicate::NE
                        };
                        let cmp = self.builder.build_int_compare(pred, *l, *r, "cmp")?;
                        Ok(CompiledValue::Bool(cmp))
                    }
                    (CompiledValue::Int64(l), CompiledValue::Int64(r)) => {
                        let pred = if id.as_str() == "==" {
                            IntPredicate::EQ
                        } else {
                            IntPredicate::NE
                        };
                        let cmp = self.builder.build_int_compare(pred, *l, *r, "cmp")?;
                        Ok(CompiledValue::Bool(cmp))
                    }
                    _ => Err(unknown_error("Type mismatch for == operation")),
                }
            }
            _ => Err(CodegenError::OpNotDefined(id.clone())),
        }
    }

    pub fn codegen_module_def(&mut self, _mod_def: &ModuleDef) -> CodegenResult<()> {
        Ok(())
    }

    pub fn codegen_type_def(&mut self, _mod_def: &TypeDef) -> CodegenResult<()> {
        todo!()
    }

    pub fn codegen_lit_expr(&mut self, literal: &Literal) -> CodegenResult<CompiledValue<'ctx>> {
        match literal {
            Literal::Number(num) => {
                let float_val = val::number_to_f64(num).unwrap();
                Ok(CompiledValue::Float(
                    self.context.f32_type().const_float(float_val),
                ))
            }
            Literal::Bool(b) => {
                if *b {
                    Ok(CompiledValue::Bool(self.bool_type().const_int(1, false)))
                } else {
                    Ok(CompiledValue::Bool(self.bool_type().const_int(0, false)))
                }
            }
            Literal::String(s) => {
                let ptr = self.builder.build_global_string_ptr(s, ".str")?;
                Ok(CompiledValue::String(ptr.as_pointer_value()))
            }
            _ => Err(unknown_error("Literal not yet implemented!")),
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

    pub fn codegen_expr(&mut self, expr: &Expression) -> CodegenResult<CompiledValue<'ctx>> {
        match expr {
            Expression::Commented(_c, e) => self.codegen_expr(e),
            Expression::Lit(lit_expr) => self.codegen_lit_expr(lit_expr),
            Expression::FnCall(fn_call) => self.codegen_fn_call(fn_call),
            Expression::OpCall(id, left, right) => self.codegen_op_call(id, left, right),

            Expression::Var(id) => match self.lookup_var(id) {
                Some(var) => {
                    let f32_type = self.context.f32_type();
                    Ok(CompiledValue::Float(
                        self.builder
                            .build_load(f32_type, *var, "load_var")?
                            .into_float_value(),
                    ))
                }
                None => self.codegen_fn_call(&FnCall::new(id.into(), FnCallArgs::empty())),
            },

            Expression::ConstOrTypeRef(_id) => todo!(),
            Expression::DBTypeRef(_id) => todo!(),
            Expression::PropFnRef(_id) => todo!(),
            Expression::EdgeProp(_id, _edge) => todo!(),
            Expression::IfElse(if_else) => self.codegen_if_else(if_else),
            Expression::Let(let_expr) => {
                let f32_type = self.context.f32_type();

                for (var_id, var_expr) in let_expr.bindings.iter() {
                    let compiled_val = self.codegen_expr(var_expr)?;
                    let alloca = self.create_entry_block_alloca(f32_type, var_id.as_str());
                    self.builder
                        .build_store(alloca, compiled_val.into_basic_value())?;
                    self.store_var(var_id.as_str(), alloca);
                }

                self.codegen_expr(&let_expr.body)
            }
            Expression::Lambda(_lambda) => todo!(),
            Expression::Query(_query) => todo!(),
            Expression::Symbol(_id) => todo!(),
            Expression::Quoted(_expr) => todo!(),
            Expression::QuotedAST(_ast) => todo!(),
            Expression::Unquoted(_expr) => todo!(),
            Expression::UnquotedAST(_ast) => todo!(),
            Expression::InlineFnDef(fn_def) => {
                self.codegen_fn_def(&fn_def.borrow())?;
                Ok(CompiledValue::Float(self.context.f32_type().const_zero()))
            }
        }
    }

    fn codegen_if_else(&mut self, if_else: &IfElse) -> CodegenResult<CompiledValue<'ctx>> {
        let cond = self.codegen_expr(&if_else.condition)?;

        let cmp = match &cond {
            CompiledValue::Float(f) => {
                let zero = self.context.f32_type().const_zero();
                Some(self.builder.build_float_compare(
                    inkwell::FloatPredicate::ONE,
                    *f,
                    zero,
                    "ifcond",
                )?)
            }
            CompiledValue::Int32(i) => {
                let zero = self.i32_type().const_int(0, false);
                Some(
                    self.builder
                        .build_int_compare(IntPredicate::NE, *i, zero, "ifcond")?,
                )
            }
            CompiledValue::Int64(i) => {
                let zero = self.i64_type().const_int(0, false);
                Some(
                    self.builder
                        .build_int_compare(IntPredicate::NE, *i, zero, "ifcond")?,
                )
            }
            CompiledValue::Bool(b) => Some(*b), // i1 can be used directly as condition
            _ => None,
        };

        let cmp = cmp.ok_or_else(|| unknown_error("Invalid condition type for if"))?;

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
            .build_conditional_branch(cmp, then_block, else_block)?;

        self.builder.position_at_end(then_block);
        let then_value = self.codegen_expr(&if_else.then_expr)?;
        let then_basic = then_value.as_basic_value();
        self.builder.build_unconditional_branch(merge_block)?;

        self.builder.position_at_end(else_block);
        let else_value = self.codegen_expr(&if_else.else_expr)?;
        let else_basic = else_value.as_basic_value();
        self.builder.build_unconditional_branch(merge_block)?;

        self.builder.position_at_end(merge_block);

        match (&then_value, &else_value) {
            (CompiledValue::Float(_), CompiledValue::Float(_)) => {
                let phi = self.builder.build_phi(self.context.f32_type(), "if-phi")?;
                phi.add_incoming(&[(&then_basic, then_block), (&else_basic, else_block)]);
                Ok(CompiledValue::Float(
                    phi.as_basic_value().into_float_value(),
                ))
            }
            (CompiledValue::Int32(_), CompiledValue::Int32(_)) => {
                let phi = self.builder.build_phi(self.i32_type(), "if-phi")?;
                phi.add_incoming(&[(&then_basic, then_block), (&else_basic, else_block)]);
                Ok(CompiledValue::Int32(phi.as_basic_value().into_int_value()))
            }
            (CompiledValue::Int64(_), CompiledValue::Int64(_)) => {
                let phi = self.builder.build_phi(self.i64_type(), "if-phi")?;
                phi.add_incoming(&[(&then_basic, then_block), (&else_basic, else_block)]);
                Ok(CompiledValue::Int64(phi.as_basic_value().into_int_value()))
            }
            _ => Err(unknown_error("Type mismatch in if-else branches")),
        }
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

        builder.build_alloca(ty, name).unwrap()
    }

    pub fn emit_ir_to_string(&self) -> String {
        self.module.print_to_string().to_string()
    }

    pub fn write_ir_to_file(&self, path: &std::path::Path) -> CodegenResult<()> {
        let ir = self.emit_ir_to_string();
        std::fs::write(path, ir).map_err(|e| unknown_error(format!("Failed to write IR: {e}")))
    }

    pub fn write_bitcode_to_file(&self, path: &std::path::Path) -> CodegenResult<()> {
        if !self.module.write_bitcode_to_path(path) {
            return Err(unknown_error("Failed to write bitcode"));
        }
        Ok(())
    }

    pub fn write_object_to_file(&self, path: &std::path::Path) -> CodegenResult<()> {
        self.target_machine
            .write_to_file(self.module, FileType::Object, path)
            .map_err(|e| unknown_error(format!("Failed to write object file: {e}")))
    }

    pub fn write_assembly_to_file(&self, path: &std::path::Path) -> CodegenResult<()> {
        self.target_machine
            .write_to_file(self.module, FileType::Assembly, path)
            .map_err(|e| unknown_error(format!("Failed to write assembly: {e}")))
    }

    pub fn context(&self) -> &'ctx Context {
        self.context
    }
}
