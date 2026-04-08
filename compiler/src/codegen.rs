use inkwell::{
    builder::{Builder, BuilderError},
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
use rogato_common::span::Span;
use rogato_common::{
    ast::{
        expression::{ExprKind, Expression},
        fn_call::{FnCall, FnCallArgs},
        fn_def::{FnDef, FnDefArgs, FnDefBody, FnDefVariant, FnDefVariants},
        if_else::IfElse,
        lambda::Lambda,
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
use rogato_common::ast::free_vars::collect_free_vars;

/// Metadata about a compiled lambda/closure, used for calling it later.
#[derive(Debug, Clone)]
pub struct LambdaCallInfo {
    pub arg_count: usize,
    pub capture_count: usize,
    pub capture_types: Vec<CompiledType>,
    pub arg_types: Vec<CompiledType>,
    pub return_type: CompiledType,
}

#[derive(Debug, Clone)]
pub enum CompiledValue<'ctx> {
    Float(FloatValue<'ctx>),
    Int32(IntValue<'ctx>),
    Int64(IntValue<'ctx>),
    String(PointerValue<'ctx>),
    Bool(IntValue<'ctx>),
    Lambda(PointerValue<'ctx>, LambdaCallInfo),
}

impl<'ctx> CompiledValue<'ctx> {
    pub fn into_basic_value(self) -> BasicValueEnum<'ctx> {
        match self {
            CompiledValue::Float(v) => v.into(),
            CompiledValue::Int32(v) => v.into(),
            CompiledValue::Int64(v) => v.into(),
            CompiledValue::String(v) => v.into(),
            CompiledValue::Bool(v) => v.into(),
            CompiledValue::Lambda(v, _) => v.into(),
        }
    }

    pub fn as_basic_value(&self) -> BasicValueEnum<'ctx> {
        match self {
            CompiledValue::Float(v) => (*v).into(),
            CompiledValue::Int32(v) => (*v).into(),
            CompiledValue::Int64(v) => (*v).into(),
            CompiledValue::String(v) => (*v).into(),
            CompiledValue::Bool(v) => (*v).into(),
            CompiledValue::Lambda(v, _) => (*v).into(),
        }
    }

    pub fn get_type(&self) -> CompiledType {
        match self {
            CompiledValue::Float(_) => CompiledType::Float,
            CompiledValue::Int32(_) => CompiledType::Int32,
            CompiledValue::Int64(_) => CompiledType::Int64,
            CompiledValue::String(_) => CompiledType::String,
            CompiledValue::Bool(_) => CompiledType::Bool,
            CompiledValue::Lambda(_, _) => CompiledType::Lambda,
        }
    }

    /// Converts a `BasicValueEnum` into a `CompiledValue` using the given `CompiledType`
    /// to determine which variant to construct. Panics for `CompiledType::Lambda` since
    /// lambda values require additional `LambdaCallInfo` metadata.
    pub fn from_basic_value(value: BasicValueEnum<'ctx>, compiled_type: CompiledType) -> Self {
        match compiled_type {
            CompiledType::Float => CompiledValue::Float(value.into_float_value()),
            CompiledType::Int32 => CompiledValue::Int32(value.into_int_value()),
            CompiledType::Int64 => CompiledValue::Int64(value.into_int_value()),
            CompiledType::String => CompiledValue::String(value.into_pointer_value()),
            CompiledType::Bool => CompiledValue::Bool(value.into_int_value()),
            CompiledType::Lambda => {
                panic!("Cannot create Lambda CompiledValue without LambdaCallInfo; use CompiledValue::Lambda directly")
            }
        }
    }
}

pub type CodegenResult<T> = Result<T, CodegenError>;

#[inline]
fn unknown_error<S: Into<String>>(message: S) -> CodegenError {
    CodegenError::Unknown(message.into())
}

/// Returns true if the expression is exactly `Var(name)`.
fn is_var_expr(var_name: &str, expr: &Expression) -> bool {
    matches!(&expr.kind, ExprKind::Var(id) if id.as_str() == var_name)
}

/// Checks if a function definition is a "lambda wrapper" — a 0-arg function whose body
/// is a single lambda expression with no captures. Returns the lambda reference if so.
///
/// This allows `let double = (x -> x * 2)` to be compiled identically to `let double x = x * 2`,
/// so that direct calls like `double 3` work correctly at the LLVM level.
fn unwrap_lambda_fn_def(fn_def: &FnDef) -> Option<&Rc<Lambda>> {
    let variant = fn_def.get_variant(0)?;
    let FnDefVariant(args, body, _) = variant;
    if !args.is_empty() {
        return None;
    }
    if let FnDefBody::RogatoFn(expr) = body.as_ref() {
        if let ExprKind::Lambda(lambda) = &expr.kind {
            let free_vars = collect_free_vars(lambda);
            if free_vars.is_empty() {
                return Some(lambda);
            }
        }
    }
    None
}

/// Checks if a function definition with N>0 args has a lambda as its body.
/// If so, returns the function's own args and the lambda reference so that the
/// caller can "flatten" the definition — merging fn args with lambda args.
///
/// This allows `let f x = y -> body` to be compiled as `let f x y = body`,
/// so that direct calls like `f 3 2` work correctly at the LLVM level.
fn flatten_lambda_fn_def(fn_def: &FnDef) -> Option<(&FnDefArgs, &Rc<Lambda>)> {
    let variant = fn_def.get_variant(0)?;
    let FnDefVariant(args, body, _) = variant;
    if args.is_empty() {
        return None; // 0-arg case is handled by unwrap_lambda_fn_def
    }
    if let FnDefBody::RogatoFn(expr) = body.as_ref() {
        if let ExprKind::Lambda(lambda) = &expr.kind {
            return Some((args, lambda));
        }
    }
    None
}

/// Infers the type of a variable by analyzing how it's used in an expression body.
/// - Used as condition in if-else → Bool
/// - Used as operand in arithmetic ops (+, -, *, /, %) → Float
/// - Used as operand in comparison ops (>, <, >=, <=, ==, !=) → Float
/// - Used as operand in boolean ops (&&, ||) → Bool
/// - Otherwise → Float (safe default)
fn infer_var_type_from_body(var_name: &str, expr: &Expression) -> CompiledType {
    match &expr.kind {
        // If the variable is used directly as the condition of an if-else → Bool
        ExprKind::IfElse(if_else) => {
            if is_var_expr(var_name, &if_else.condition) {
                return CompiledType::Bool;
            }
            // Recurse into branches
            let from_cond = infer_var_type_from_body(var_name, &if_else.condition);
            if from_cond != CompiledType::Float {
                return from_cond;
            }
            let from_then = infer_var_type_from_body(var_name, &if_else.then_expr);
            if from_then != CompiledType::Float {
                return from_then;
            }
            infer_var_type_from_body(var_name, &if_else.else_expr)
        }
        // Variable used in arithmetic → Float
        ExprKind::OpCall(op, left, right) => {
            let is_arithmetic = matches!(op.as_str(), "+" | "-" | "*" | "/" | "%");
            let is_comparison = matches!(op.as_str(), ">" | "<" | ">=" | "<=" | "==" | "!=");
            if (is_arithmetic || is_comparison)
                && (is_var_expr(var_name, left) || is_var_expr(var_name, right))
            {
                return CompiledType::Float;
            }
            // Check if used in boolean ops as operand → Bool
            let is_boolean_op = matches!(op.as_str(), "&&" | "||");
            if is_boolean_op && (is_var_expr(var_name, left) || is_var_expr(var_name, right)) {
                return CompiledType::Bool;
            }
            // Recurse
            let from_left = infer_var_type_from_body(var_name, left);
            if from_left != CompiledType::Float {
                return from_left;
            }
            infer_var_type_from_body(var_name, right)
        }
        ExprKind::Let(let_expr) => {
            // Check bindings - if the var is rebound, stop (shadowed)
            for (binding_id, binding_expr) in let_expr.bindings.iter() {
                if binding_id.as_str() == var_name {
                    return CompiledType::Float; // shadowed, can't infer further
                }
                let from_binding = infer_var_type_from_body(var_name, binding_expr);
                if from_binding != CompiledType::Float {
                    return from_binding;
                }
            }
            infer_var_type_from_body(var_name, &let_expr.body)
        }
        ExprKind::FnCall(fn_call) => {
            // Recurse into fn call args
            for arg in fn_call.args.iter() {
                let from_arg = infer_var_type_from_body(var_name, arg);
                if from_arg != CompiledType::Float {
                    return from_arg;
                }
            }
            CompiledType::Float
        }
        ExprKind::Commented(_, inner) => infer_var_type_from_body(var_name, inner),
        ExprKind::Lambda(_) => CompiledType::Float, // don't recurse into lambda bodies (different scope)
        _ => CompiledType::Float,
    }
}

/// Infers the compiled types for each argument from patterns and a body expression.
/// 1. Literal patterns (Number, Bool, String, Symbol) directly indicate the type.
/// 2. For `Var` patterns, analyzes how the variable is used in the body.
/// 3. Falls back to `Float` if the type cannot be determined.
fn infer_arg_types_from_patterns<'a>(
    patterns: impl Iterator<Item = &'a Rc<Pattern>>,
    body: &Expression,
) -> Vec<CompiledType> {
    patterns
        .map(|pattern| match pattern.as_ref() {
            Pattern::Number(_) => CompiledType::Float,
            Pattern::Bool(_) => CompiledType::Bool,
            Pattern::String(_) => CompiledType::String,
            Pattern::Symbol(_) => CompiledType::String,
            Pattern::Var(var_id) => infer_var_type_from_body(var_id.as_str(), body),
            // Complex patterns that represent container types should be treated as pointers
            Pattern::ListCons(_, _) => CompiledType::Lambda,
            Pattern::EmptyList => CompiledType::Lambda,
            Pattern::List(_) => CompiledType::Lambda,
            Pattern::Tuple(_, _) => CompiledType::Lambda,
            Pattern::Map(_) => CompiledType::Lambda,
            Pattern::MapCons(_, _) => CompiledType::Lambda,
            _ => CompiledType::Float,
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompiledType {
    Float,
    Int32,
    Int64,
    String,
    Bool,
    Lambda,
}

impl<'ctx> CompiledType {
    pub fn from_type_expression(te: &TypeExpression) -> Self {
        match te {
            TypeExpression::FunctionType(_, _) => CompiledType::Lambda,
            TypeExpression::Int32Type => CompiledType::Int32,
            TypeExpression::Int64Type => CompiledType::Int64,
            TypeExpression::StringType | TypeExpression::SymbolType => CompiledType::String,
            TypeExpression::BoolType => CompiledType::Bool,
            TypeExpression::NumberType => CompiledType::Float,
            // Compound/container types are represented as pointers (like Lambda)
            TypeExpression::ListType(_)
            | TypeExpression::SetType(_)
            | TypeExpression::MapType(_, _)
            | TypeExpression::VectorType(_)
            | TypeExpression::StackType(_)
            | TypeExpression::QueueType(_)
            | TypeExpression::StructType(_)
            | TypeExpression::TupleType(_) => CompiledType::Lambda, // pointer type
            TypeExpression::TypeRef(_) => CompiledType::Float, // fallback for now
            TypeExpression::Unknown => CompiledType::Float,    // fallback
        }
    }

    pub fn as_basic_type_enum(&self, ctx: &'ctx Context) -> BasicTypeEnum<'ctx> {
        match self {
            CompiledType::Float => ctx.f32_type().into(),
            CompiledType::Int32 => ctx.i32_type().into(),
            CompiledType::Int64 => ctx.i64_type().into(),
            CompiledType::String => ctx.ptr_type(AddressSpace::default()).into(),
            CompiledType::Bool => ctx.bool_type().into(),
            CompiledType::Lambda => ctx.ptr_type(AddressSpace::default()).into(),
        }
    }

    pub fn as_metadata_type_enum(&self, ctx: &'ctx Context) -> BasicMetadataTypeEnum<'ctx> {
        match self {
            CompiledType::Float => BasicMetadataTypeEnum::FloatType(ctx.f32_type()),
            CompiledType::Int32 => BasicMetadataTypeEnum::IntType(ctx.i32_type()),
            CompiledType::Int64 => BasicMetadataTypeEnum::IntType(ctx.i64_type()),
            CompiledType::String => {
                BasicMetadataTypeEnum::PointerType(ctx.ptr_type(AddressSpace::default()))
            }
            CompiledType::Bool => BasicMetadataTypeEnum::IntType(ctx.bool_type()),
            CompiledType::Lambda => {
                BasicMetadataTypeEnum::PointerType(ctx.ptr_type(AddressSpace::default()))
            }
        }
    }

    /// Determines the `CompiledType` from an LLVM `BasicTypeEnum`.
    /// Uses bit-width to distinguish between Bool (i1), Int32 (i32), and Int64 (i64).
    /// Pointer types default to `String` (could also be `Lambda` — callers should
    /// use additional context to distinguish when needed).
    pub fn from_basic_type_enum(ty: BasicTypeEnum<'ctx>) -> Self {
        match ty {
            BasicTypeEnum::FloatType(_) => CompiledType::Float,
            BasicTypeEnum::IntType(it) => match it.get_bit_width() {
                1 => CompiledType::Bool,
                64 => CompiledType::Int64,
                _ => CompiledType::Int32,
            },
            BasicTypeEnum::PointerType(_) => CompiledType::String,
            _ => CompiledType::Float, // fallback for array, struct, vector types
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
    variable_scopes: Vec<HashMap<String, (PointerValue<'ctx>, CompiledType)>>,
    lambda_info: HashMap<String, LambdaCallInfo>,
    lambda_counter: usize,
    printf: Option<FunctionValue<'ctx>>,
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
            variable_scopes: vec![HashMap::new()],
            lambda_info: HashMap::new(),
            lambda_counter: 0,
            printf: None,
        }
    }

    pub fn init_stdlib(&mut self) {
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let printf_type = ptr_type.fn_type(&[BasicMetadataTypeEnum::PointerType(ptr_type)], true);
        let printf = self.module.add_function("printf", printf_type, None);
        self.printf = Some(printf);

        // Declare runtime helper functions for list pattern matching
        let i8_type = self.context.i8_type();
        let i32_type = self.context.i32_type();

        // rogato_list_is_empty(list_ptr) -> i8 (returns 1 if empty, 0 otherwise)
        let list_is_empty_type = i8_type.fn_type(&[ptr_type.into()], false);
        self.module
            .add_function("rogato_list_is_empty", list_is_empty_type, None);

        // rogato_list_head(list_ptr) -> ValueRef (returns head element or null pointer)
        let list_head_type = ptr_type.fn_type(&[ptr_type.into()], false);
        self.module
            .add_function("rogato_list_head", list_head_type, None);

        // rogato_list_tail(list_ptr) -> ValueRef (returns tail list or null pointer)
        let list_tail_type = ptr_type.fn_type(&[ptr_type.into()], false);
        self.module
            .add_function("rogato_list_tail", list_tail_type, None);

        // rogato_tuple_len(tuple_ptr) -> i32 (returns tuple length)
        let tuple_len_type = i32_type.fn_type(&[ptr_type.into()], false);
        self.module
            .add_function("rogato_tuple_len", tuple_len_type, None);

        // rogato_tuple_get(tuple_ptr, index) -> ValueRef (returns tuple element at index)
        let tuple_get_type = ptr_type.fn_type(&[ptr_type.into(), i32_type.into()], false);
        self.module
            .add_function("rogato_tuple_get", tuple_get_type, None);

        // rogato_list_len(list_ptr) -> i32 (returns list length)
        let list_len_type = i32_type.fn_type(&[ptr_type.into()], false);
        self.module
            .add_function("rogato_list_len", list_len_type, None);

        // rogato_list_get(list_ptr, index) -> ValueRef (returns list element at index)
        let list_get_type = ptr_type.fn_type(&[ptr_type.into(), i32_type.into()], false);
        self.module
            .add_function("rogato_list_get", list_get_type, None);

        // rogato_map_len(map_ptr) -> i32 (returns map length)
        let map_len_type = i32_type.fn_type(&[ptr_type.into()], false);
        self.module
            .add_function("rogato_map_len", map_len_type, None);

        // rogato_map_keys(map_ptr) -> ValueRef (returns list of map keys)
        let map_keys_type = ptr_type.fn_type(&[ptr_type.into()], false);
        self.module
            .add_function("rogato_map_keys", map_keys_type, None);

        // rogato_map_get(map_ptr, key) -> ValueRef (returns value for key or null)
        let map_get_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
        self.module
            .add_function("rogato_map_get", map_get_type, None);
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
        self.context.ptr_type(AddressSpace::default())
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

        // If this is a 0-arg function wrapping a non-capturing lambda,
        // declare the function with the lambda's args instead.
        if let Some(lambda) = unwrap_lambda_fn_def(fn_def) {
            if let Some(first_lv) = lambda.variants_iter().next() {
                let first_lv = first_lv.deref();
                let return_type = self.infer_expr_type_with_checker(&first_lv.body);
                let return_llvm_type = return_type.as_basic_type_enum(self.context);
                let arg_types = infer_arg_types_from_patterns(first_lv.args.iter(), &first_lv.body);
                let fn_arg_types: Vec<BasicMetadataTypeEnum<'ctx>> = arg_types
                    .iter()
                    .map(|t| t.as_metadata_type_enum(self.context))
                    .collect();
                let fn_type = return_llvm_type.fn_type(&fn_arg_types, false);
                let func = self.module.add_function(func_name, fn_type, None);
                return Ok(func);
            }
        }

        // If this is an N-arg function whose body is a lambda expression,
        // declare the function with merged args (fn_args ++ lambda_args).
        // This flattens `let f x = y -> body` into `let f x y = body`.
        if let Some((fn_args, lambda)) = flatten_lambda_fn_def(fn_def) {
            if let Some(first_lv) = lambda.variants_iter().next() {
                let first_lv = first_lv.deref();
                let return_type = self.infer_expr_type_with_checker(&first_lv.body);
                let return_llvm_type = return_type.as_basic_type_enum(self.context);
                let arg_types = infer_arg_types_from_patterns(
                    fn_args.iter().chain(first_lv.args.iter()),
                    &first_lv.body,
                );
                let fn_arg_types: Vec<BasicMetadataTypeEnum<'ctx>> = arg_types
                    .iter()
                    .map(|t| t.as_metadata_type_enum(self.context))
                    .collect();
                let fn_type = return_llvm_type.fn_type(&fn_arg_types, false);
                let func = self.module.add_function(func_name, fn_type, None);
                return Ok(func);
            }
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

        let arg_types = match body.as_ref() {
            FnDefBody::RogatoFn(expr) => self.infer_variant_arg_types(args, expr),
            _ => args.iter().map(|_| CompiledType::Float).collect(),
        };
        let fn_arg_types: Vec<BasicMetadataTypeEnum<'ctx>> = arg_types
            .iter()
            .map(|t| t.as_metadata_type_enum(self.context))
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

    /// Infers the compiled types for each argument of a function variant.
    /// Uses pattern analysis and body expression walking:
    /// 1. Literal patterns (Number, Bool, String, Symbol) directly indicate the type.
    /// 2. For `Var` patterns, analyzes how the variable is used in the body.
    /// 3. Falls back to `Float` if the type cannot be determined.
    fn infer_variant_arg_types(&self, args: &FnDefArgs, body: &Expression) -> Vec<CompiledType> {
        infer_arg_types_from_patterns(args.iter(), body)
    }

    /// Infers argument types for a multi-variant function by merging type info across variants.
    /// Concrete patterns (Number, Bool, String) in any variant determine the type for that position.
    /// For positions with only Var/Any patterns, analyzes the last variant's body.
    fn infer_multi_variant_arg_types(&self, fn_def: &FnDef) -> Vec<CompiledType> {
        let variants: Vec<_> = fn_def.variants_iter().collect();
        if variants.is_empty() {
            return vec![];
        }
        let arg_count = variants[0].0.len();
        let mut arg_types = vec![None; arg_count];

        // Scan all variants for concrete patterns
        for variant in &variants {
            for (i, pattern) in variant.0.iter().enumerate() {
                if arg_types[i].is_some() {
                    continue;
                }
                match pattern.as_ref() {
                    Pattern::Number(_) => arg_types[i] = Some(CompiledType::Float),
                    Pattern::Bool(_) => arg_types[i] = Some(CompiledType::Bool),
                    Pattern::String(_) => arg_types[i] = Some(CompiledType::String),
                    Pattern::Symbol(_) => arg_types[i] = Some(CompiledType::String),
                    // Complex patterns that represent container types should be treated as pointers
                    Pattern::ListCons(_, _) => arg_types[i] = Some(CompiledType::Lambda),
                    Pattern::EmptyList => arg_types[i] = Some(CompiledType::Lambda),
                    Pattern::List(_) => arg_types[i] = Some(CompiledType::Lambda),
                    Pattern::Tuple(_, _) => arg_types[i] = Some(CompiledType::Lambda),
                    Pattern::Map(_) => arg_types[i] = Some(CompiledType::Lambda),
                    Pattern::MapCons(_, _) => arg_types[i] = Some(CompiledType::Lambda),
                    _ => {}
                }
            }
        }

        // For remaining unknowns, analyze the last variant's body (the catch-all)
        let last_variant = variants.last().unwrap();
        if let FnDefBody::RogatoFn(body) = last_variant.1.deref() {
            for (i, pattern) in last_variant.0.iter().enumerate() {
                if arg_types[i].is_none() {
                    match pattern.as_ref() {
                        Pattern::Var(var_id) => {
                            arg_types[i] = Some(infer_var_type_from_body(var_id.as_str(), body));
                        }
                        // For complex patterns, default to Lambda (pointer type)
                        Pattern::ListCons(_, _)
                        | Pattern::EmptyList
                        | Pattern::List(_)
                        | Pattern::Tuple(_, _)
                        | Pattern::Map(_)
                        | Pattern::MapCons(_, _) => {
                            arg_types[i] = Some(CompiledType::Lambda);
                        }
                        _ => {}
                    }
                }
            }
        }

        arg_types
            .into_iter()
            .map(|t| t.unwrap_or(CompiledType::Float))
            .collect()
    }

    pub fn codegen_fn_def(&mut self, fn_def: &FnDef) -> CodegenResult<FunctionValue<'ctx>> {
        let func_name = fn_def.id();

        let variants: Vec<_> = fn_def.variants_iter().collect();

        if variants.len() > 1 {
            return self.codegen_multi_variant_fn(fn_def);
        }

        // If this is a 0-arg function wrapping a non-capturing lambda,
        // compile the function with the lambda's args and body directly.
        if let Some(lambda) = unwrap_lambda_fn_def(fn_def) {
            if let Some(first_lv) = lambda.variants_iter().next() {
                let first_lv = first_lv.deref();
                let arg_types = infer_arg_types_from_patterns(first_lv.args.iter(), &first_lv.body);

                // Reuse the function already declared by declare_fn_signature if available,
                // otherwise create it now (e.g. when codegen_fn_def is called directly in tests).
                let func = match self.module.get_function(func_name.as_str()) {
                    Some(f) => f,
                    None => {
                        let return_type = self.infer_expr_type_with_checker(&first_lv.body);
                        let return_llvm_type = return_type.as_basic_type_enum(self.context);
                        let fn_arg_types: Vec<BasicMetadataTypeEnum<'ctx>> = arg_types
                            .iter()
                            .map(|t| t.as_metadata_type_enum(self.context))
                            .collect();
                        let fn_type = return_llvm_type.fn_type(&fn_arg_types, false);
                        self.module.add_function(func_name, fn_type, None)
                    }
                };

                return self.codegen_unwrapped_lambda_body(fn_def, func, first_lv, &arg_types);
            }
        }

        // If this is an N-arg function whose body is a lambda expression,
        // flatten it by merging fn args with lambda args and compiling the lambda body directly.
        // This compiles `let f x = y -> body` as if it were `let f x y = body`.
        if let Some((fn_args, lambda)) = flatten_lambda_fn_def(fn_def) {
            if let Some(first_lv) = lambda.variants_iter().next() {
                let first_lv = first_lv.deref();
                let arg_types = infer_arg_types_from_patterns(
                    fn_args.iter().chain(first_lv.args.iter()),
                    &first_lv.body,
                );

                let func = match self.module.get_function(func_name.as_str()) {
                    Some(f) => f,
                    None => {
                        let return_type = self.infer_expr_type_with_checker(&first_lv.body);
                        let return_llvm_type = return_type.as_basic_type_enum(self.context);
                        let fn_arg_types: Vec<BasicMetadataTypeEnum<'ctx>> = arg_types
                            .iter()
                            .map(|t| t.as_metadata_type_enum(self.context))
                            .collect();
                        let fn_type = return_llvm_type.fn_type(&fn_arg_types, false);
                        self.module.add_function(func_name, fn_type, None)
                    }
                };

                return self
                    .codegen_flattened_lambda_body(fn_def, func, fn_args, first_lv, &arg_types);
            }
        }

        let FnDefVariant(args, body, _return_type) = fn_def.get_variant(0).unwrap();

        let return_type = match _return_type {
            Some(rexpr) => CompiledType::from_type_expression(rexpr),
            None => match body.as_ref() {
                FnDefBody::RogatoFn(expr) => self.infer_expr_type_with_checker(expr),
                _ => CompiledType::Float,
            },
        };

        let return_llvm_type = return_type.as_basic_type_enum(self.context);

        let arg_types = match body.as_ref() {
            FnDefBody::RogatoFn(expr) => self.infer_variant_arg_types(args, expr),
            _ => args.iter().map(|_| CompiledType::Float).collect(),
        };
        let fn_arg_types: Vec<BasicMetadataTypeEnum<'ctx>> = arg_types
            .iter()
            .map(|t| t.as_metadata_type_enum(self.context))
            .collect();

        // Reuse the function already declared by declare_fn_signature if available,
        // otherwise create it now (e.g. when codegen_fn_def is called directly in tests).
        let func = match self.module.get_function(func_name.as_str()) {
            Some(f) => f,
            None => {
                let fn_type = return_llvm_type.fn_type(&fn_arg_types, false);
                self.module.add_function(func_name, fn_type, None)
            }
        };

        self.codegen_fn_body(fn_def, func)
    }

    /// Compiles a function whose N>0 args are merged with its lambda body's args.
    /// This is used when a function with explicit params returns a lambda, e.g.
    /// `let f x = y -> (quadruple y) + (double x)` is compiled as `let f x y = (quadruple y) + (double x)`.
    fn codegen_flattened_lambda_body(
        &mut self,
        fn_def: &FnDef,
        func: FunctionValue<'ctx>,
        fn_args: &FnDefArgs,
        lambda_variant: &rogato_common::ast::lambda::LambdaVariant,
        arg_types: &[CompiledType],
    ) -> CodegenResult<FunctionValue<'ctx>> {
        self.set_current_fn_value(func);

        let basic_block = self.context.append_basic_block(func, fn_def.id());
        self.builder.position_at_end(basic_block);

        // Merge fn_def args and lambda args into one combined parameter list
        let all_patterns: Vec<&Rc<Pattern>> =
            fn_args.iter().chain(lambda_variant.args.iter()).collect();
        let params: Vec<_> = func.get_param_iter().collect();

        for (i, pattern) in all_patterns.iter().enumerate() {
            if let Pattern::Var(var_id) = pattern.as_ref() {
                let param_type = arg_types.get(i).copied().unwrap_or(CompiledType::Float);
                let llvm_type = param_type.as_basic_type_enum(self.context);
                let alloca = self.create_entry_block_alloca(llvm_type, var_id.as_str());
                self.builder.build_store(alloca, params[i])?;
                self.store_var(var_id.as_str(), alloca, param_type);
            }
        }

        // Compile the lambda body directly (not the lambda expression itself)
        let compiled_val = self.codegen_expr(&lambda_variant.body)?;
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
            Err(CodegenError::FnDefValidationFailed(
                fn_def.id().clone(),
                None,
            ))
        }
    }

    /// Compiles a function whose body was a lambda expression, using the lambda's
    /// args and body directly as the function's parameters and implementation.
    /// This is used when a 0-arg function wraps a non-capturing lambda, e.g.
    /// `let double = (x -> x * 2)` is compiled as if it were `let double x = x * 2`.
    fn codegen_unwrapped_lambda_body(
        &mut self,
        fn_def: &FnDef,
        func: FunctionValue<'ctx>,
        lambda_variant: &rogato_common::ast::lambda::LambdaVariant,
        arg_types: &[CompiledType],
    ) -> CodegenResult<FunctionValue<'ctx>> {
        self.set_current_fn_value(func);

        let basic_block = self.context.append_basic_block(func, fn_def.id());
        self.builder.position_at_end(basic_block);

        // Store lambda params as function arguments
        let params: Vec<_> = func.get_param_iter().collect();
        for (i, pattern) in lambda_variant.args.iter().enumerate() {
            if let Pattern::Var(var_id) = pattern.as_ref() {
                let param_type = arg_types.get(i).copied().unwrap_or(CompiledType::Float);
                let llvm_type = param_type.as_basic_type_enum(self.context);
                let alloca = self.create_entry_block_alloca(llvm_type, var_id.as_str());
                self.builder.build_store(alloca, params[i])?;
                self.store_var(var_id.as_str(), alloca, param_type);
            }
        }

        // Compile the lambda body directly
        let compiled_val = self.codegen_expr(&lambda_variant.body)?;
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
            Err(CodegenError::FnDefValidationFailed(
                fn_def.id().clone(),
                None,
            ))
        }
    }

    fn codegen_fn_body(
        &mut self,
        fn_def: &FnDef,
        func: FunctionValue<'ctx>,
    ) -> CodegenResult<FunctionValue<'ctx>> {
        let FnDefVariant(args, body, _return_type) = fn_def.get_variant(0).unwrap();

        self.set_current_fn_value(func);

        let basic_block = self.context.append_basic_block(func, fn_def.id());
        self.builder.position_at_end(basic_block);

        let params: Vec<_> = func.get_param_iter().collect();
        self.store_pattern_args(args, &params)?;

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
                    Err(CodegenError::FnDefValidationFailed(
                        fn_def.id().clone(),
                        None,
                    ))
                }
            }
            _ => Err(unknown_error("Cannot compile function with NativeFn body!")),
        }
    }

    /// Stores function/lambda arguments into stack allocas based on pattern matching.
    /// Handles variable patterns, literal patterns (handled by variant condition matching),
    /// and complex patterns like ListCons that require runtime helpers.
    fn store_pattern_args(
        &mut self,
        args: &FnDefArgs,
        params: &[BasicValueEnum<'ctx>],
    ) -> CodegenResult<()> {
        for (i, arg_pattern) in args.iter().enumerate() {
            match arg_pattern.as_ref() {
                Pattern::Var(var_id) => {
                    let param_type = CompiledType::from_basic_type_enum(params[i].get_type());
                    let llvm_type = param_type.as_basic_type_enum(self.context);
                    let alloca = self.create_entry_block_alloca(llvm_type, var_id.as_str());
                    self.builder.build_store(alloca, params[i])?;
                    self.store_var(var_id.as_str(), alloca, param_type);
                }
                // Literal and wildcard patterns are already handled by variant condition matching
                Pattern::Number(_)
                | Pattern::Bool(_)
                | Pattern::String(_)
                | Pattern::Symbol(_)
                | Pattern::Any => {}
                // List cons pattern: [head :: tail] - extract head and tail from list
                Pattern::ListCons(head_pattern, tail_pattern) => {
                    self.codegen_store_list_cons_pattern(
                        arg_pattern.as_ref(),
                        params[i].into_pointer_value(),
                    )?;
                }
                // Empty list pattern: []
                Pattern::EmptyList => {
                    // Just verify the list is empty at runtime (for validation)
                    // If not empty, this variant won't match (handled by multi-variant fallback or error)
                }
                // List pattern: [a, b, c] - match against list of specific length
                Pattern::List(patterns) => {
                    self.codegen_store_list_pattern(
                        arg_pattern.as_ref(),
                        params[i].into_pointer_value(),
                    )?;
                }
                // Tuple pattern: {a, b, c}
                Pattern::Tuple(len, patterns) => {
                    self.codegen_store_tuple_pattern(
                        arg_pattern.as_ref(),
                        params[i].into_pointer_value(),
                    )?;
                }
                // Map pattern: {key1: val1, key2: val2}
                Pattern::Map(kv_pairs) => {
                    self.codegen_store_map_pattern(
                        arg_pattern.as_ref(),
                        params[i].into_pointer_value(),
                    )?;
                }
                // Map cons pattern: {key1: val1, key2: val2 :: rest}
                Pattern::MapCons(kv_pairs, rest_pattern) => {
                    self.codegen_store_map_cons_pattern(
                        arg_pattern.as_ref(),
                        params[i].into_pointer_value(),
                    )?;
                }
            }
        }
        Ok(())
    }

    /// Generates code to store list cons pattern [head :: tail] = value
    fn codegen_store_list_cons_pattern(
        &mut self,
        pattern: &Pattern,
        list_ptr: PointerValue<'ctx>,
    ) -> CodegenResult<()> {
        // For Pattern::ListCons, we need to extract head and tail
        if let Pattern::ListCons(head_pattern, tail_pattern) = pattern {
            let ptr_type = self.context.ptr_type(AddressSpace::default());

            // Call rogato_list_head(list_ptr)
            let list_head_fn = self
                .module
                .get_function("rogato_list_head")
                .ok_or_else(|| unknown_error("rogato_list_head not found"))?;
            let head_ptr = self
                .builder
                .build_call(list_head_fn, &[list_ptr.into()], "list_head_result")?
                .try_as_basic_value()
                .basic()
                .ok_or_else(|| unknown_error("Invalid call produced"))?;

            // Call rogato_list_tail(list_ptr)
            let list_tail_fn = self
                .module
                .get_function("rogato_list_tail")
                .ok_or_else(|| unknown_error("rogato_list_tail not found"))?;
            let tail_ptr = self
                .builder
                .build_call(list_tail_fn, &[list_ptr.into()], "list_tail_result")?
                .try_as_basic_value()
                .basic()
                .ok_or_else(|| unknown_error("Invalid call produced"))?;

            // Store head value
            match head_pattern.as_ref() {
                Pattern::Var(var_id) => {
                    let alloca = self.create_entry_block_alloca(ptr_type, var_id.as_str());
                    self.builder.build_store(alloca, head_ptr)?;
                    // Use String type for generic pointer values (list elements)
                    self.store_var(var_id.as_str(), alloca, CompiledType::String);
                }
                _ => {
                    // For non-var head patterns, just validate (already handled in variant matching)
                }
            }

            // Store tail value
            match tail_pattern.as_ref() {
                Pattern::Var(var_id) => {
                    let alloca = self.create_entry_block_alloca(ptr_type, var_id.as_str());
                    self.builder.build_store(alloca, tail_ptr)?;
                    // Use String type for generic pointer values (list elements)
                    self.store_var(var_id.as_str(), alloca, CompiledType::String);
                }
                _ => {
                    // For non-var tail patterns, just validate
                }
            }
        }

        Ok(())
    }

    /// Generates code to store list pattern [a, b, c] = value
    fn codegen_store_list_pattern(
        &mut self,
        pattern: &Pattern,
        list_ptr: PointerValue<'ctx>,
    ) -> CodegenResult<()> {
        if let Pattern::List(patterns) = pattern {
            let ptr_type = self.context.ptr_type(AddressSpace::default());
            let i32_type = self.i32_type();

            // Check list length matches
            let list_len_fn = self
                .module
                .get_function("rogato_list_len")
                .ok_or_else(|| unknown_error("rogato_list_len not found"))?;
            let list_len = self
                .builder
                .build_call(list_len_fn, &[list_ptr.into()], "list_len_result")?
                .try_as_basic_value()
                .basic()
                .ok_or_else(|| unknown_error("Invalid call produced"))?
                .into_int_value();
            let expected_len = i32_type.const_int(patterns.len() as u64, false);
            let _len_matches = self.builder.build_int_compare(
                IntPredicate::EQ,
                list_len,
                expected_len,
                "list_len_check",
            )?;

            // Store each element
            for (idx, elem_pattern) in patterns.iter().enumerate() {
                match elem_pattern.as_ref() {
                    Pattern::Var(var_id) => {
                        // Call rogato_list_get(list_ptr, idx)
                        let list_get_fn = self
                            .module
                            .get_function("rogato_list_get")
                            .ok_or_else(|| unknown_error("rogato_list_get not found"))?;
                        let idx_val = i32_type.const_int(idx as u64, false);
                        let elem_ptr = self
                            .builder
                            .build_call(
                                list_get_fn,
                                &[list_ptr.into(), idx_val.into()],
                                "list_get_result",
                            )?
                            .try_as_basic_value()
                            .basic()
                            .ok_or_else(|| unknown_error("Invalid call produced"))?;

                        let alloca = self.create_entry_block_alloca(ptr_type, var_id.as_str());
                        self.builder.build_store(alloca, elem_ptr)?;
                        // Use String type for generic pointer values (list elements)
                        self.store_var(var_id.as_str(), alloca, CompiledType::String);
                    }
                    _ => {
                        // For literal patterns in list, we don't bind variables here
                        // (handled by variant condition matching for multi-variant functions)
                    }
                }
            }
        }

        Ok(())
    }

    /// Generates code to store tuple pattern {a, b, c} = value
    fn codegen_store_tuple_pattern(
        &mut self,
        pattern: &Pattern,
        tuple_ptr: PointerValue<'ctx>,
    ) -> CodegenResult<()> {
        if let Pattern::Tuple(_len, patterns) = pattern {
            let ptr_type = self.context.ptr_type(AddressSpace::default());
            let i32_type = self.i32_type();

            // Check tuple length matches
            let tuple_len_fn = self
                .module
                .get_function("rogato_tuple_len")
                .ok_or_else(|| unknown_error("rogato_tuple_len not found"))?;
            let tuple_len = self
                .builder
                .build_call(tuple_len_fn, &[tuple_ptr.into()], "tuple_len_result")?
                .try_as_basic_value()
                .basic()
                .ok_or_else(|| unknown_error("Invalid call produced"))?
                .into_int_value();
            let expected_len = i32_type.const_int(patterns.len() as u64, false);
            let _len_matches = self.builder.build_int_compare(
                IntPredicate::EQ,
                tuple_len,
                expected_len,
                "tuple_len_check",
            )?;

            // Store each element
            for (idx, elem_pattern) in patterns.iter().enumerate() {
                match elem_pattern.as_ref() {
                    Pattern::Var(var_id) => {
                        // Call rogato_tuple_get(tuple_ptr, idx)
                        let tuple_get_fn = self
                            .module
                            .get_function("rogato_tuple_get")
                            .ok_or_else(|| unknown_error("rogato_tuple_get not found"))?;
                        let idx_val = i32_type.const_int(idx as u64, false);
                        let elem_ptr = self
                            .builder
                            .build_call(
                                tuple_get_fn,
                                &[tuple_ptr.into(), idx_val.into()],
                                "tuple_get_result",
                            )?
                            .try_as_basic_value()
                            .basic()
                            .ok_or_else(|| unknown_error("Invalid call produced"))?;

                        let alloca = self.create_entry_block_alloca(ptr_type, var_id.as_str());
                        self.builder.build_store(alloca, elem_ptr)?;
                        // Use String type for generic pointer values (tuple elements)
                        self.store_var(var_id.as_str(), alloca, CompiledType::String);
                    }
                    _ => {
                        // For literal patterns in tuple
                    }
                }
            }
        }

        Ok(())
    }

    /// Generates code to store map pattern {key1: val1, key2: val2} = value
    fn codegen_store_map_pattern(
        &mut self,
        pattern: &Pattern,
        map_ptr: PointerValue<'ctx>,
    ) -> CodegenResult<()> {
        if let Pattern::Map(kv_pairs) = pattern {
            let ptr_type = self.context.ptr_type(AddressSpace::default());
            let i32_type = self.i32_type();

            // Check map length matches
            let map_len_fn = self
                .module
                .get_function("rogato_map_len")
                .ok_or_else(|| unknown_error("rogato_map_len not found"))?;
            let map_len = self
                .builder
                .build_call(map_len_fn, &[map_ptr.into()], "map_len_result")?
                .try_as_basic_value()
                .basic()
                .ok_or_else(|| unknown_error("Invalid call produced"))?
                .into_int_value();
            let expected_len = i32_type.const_int(kv_pairs.len() as u64, false);
            let _len_matches = self.builder.build_int_compare(
                IntPredicate::EQ,
                map_len,
                expected_len,
                "map_len_check",
            )?;

            // If length doesn't match, this pattern won't bind variables
            // Store each key-value pair
            for kv_pair in kv_pairs.iter() {
                let (key_pattern, val_pattern) = kv_pair.pair();
                match val_pattern.as_ref() {
                    Pattern::Var(var_id) => {
                        // For map patterns with var values, we need to get the value
                        // This would require more complex runtime support for key matching
                        // For now, skip binding - literal keys are handled by condition matching
                    }
                    _ => {
                        // For non-var values in map patterns
                    }
                }
            }
        }

        Ok(())
    }

    /// Generates code to store map cons pattern {key1: val1, key2: val2 :: rest} = value
    fn codegen_store_map_cons_pattern(
        &mut self,
        pattern: &Pattern,
        map_ptr: PointerValue<'ctx>,
    ) -> CodegenResult<()> {
        if let Pattern::MapCons(kv_pairs, rest_pattern) = pattern {
            // For map cons patterns, we extract values for specific keys
            // and bind the rest to a variable if present
            match rest_pattern.as_ref() {
                Pattern::Var(var_id) => {
                    // If there's a rest pattern, bind the remaining map
                    let ptr_type = self.context.ptr_type(AddressSpace::default());
                    let alloca = self.create_entry_block_alloca(ptr_type, var_id.as_str());
                    // For now, bind the entire map as-is
                    self.builder.build_store(alloca, map_ptr)?;
                    // Use String type for generic pointer values (map)
                    self.store_var(var_id.as_str(), alloca, CompiledType::String);
                }
                _ => {
                    // No rest pattern to bind
                }
            }
        }

        Ok(())
    }

    fn codegen_multi_variant_fn(&mut self, fn_def: &FnDef) -> CodegenResult<FunctionValue<'ctx>> {
        let variants: Vec<_> = fn_def.variants_iter().collect();

        if variants.is_empty() {
            return Err(CodegenError::FnDefValidationFailed(
                fn_def.id().clone(),
                None,
            ));
        }

        let first_variant = &variants[0];
        let arg_count = first_variant.0.len();

        for variant in variants.iter() {
            if variant.0.len() != arg_count {
                return Err(CodegenError::FnDefValidationFailed(
                    fn_def.id().clone(),
                    None,
                ));
            }
        }

        // Check if any variant has at least one position with a catch-all pattern (Var or Any)
        // This is required for multi-variant functions to ensure all inputs are covered
        let has_catch_all = variants
            .iter()
            .any(|v| {
                v.0.iter()
                    .any(|p| matches!(p.deref(), Pattern::Var(_) | Pattern::Any))
            });

        // Only require catch-all if all variants have literal-only patterns (no Var/Any at any position)
        // This allows partial matching where some variants have literal patterns and others are catch-all
        let all_literal = variants.iter().all(|v| {
            v.0.iter().all(|p| {
                matches!(
                    p.deref(),
                    Pattern::Number(_) | Pattern::Bool(_) | Pattern::String(_)
                        | Pattern::Symbol(_) | Pattern::EmptyList
                )
            })
        });

        // Also check if any position has variable patterns that can match anything
        let has_var_pattern = variants.iter().any(|v| {
            v.0.iter()
                .any(|p| matches!(p.deref(), Pattern::Var(_) | Pattern::Any))
        });

        // Require catch-all only if:
        // 1. All variants have literal-only patterns (potential non-exhaustive), AND
        // 2. No variant has any variable pattern that could match anything
        if all_literal && !has_var_pattern {
            return Err(CodegenError::FnPatternUncovered(fn_def.id().clone(), None));
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

        let arg_types = self.infer_multi_variant_arg_types(fn_def);
        let fn_arg_types: Vec<BasicMetadataTypeEnum<'ctx>> = arg_types
            .iter()
            .map(|t| t.as_metadata_type_enum(self.context))
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
            Err(CodegenError::FnDefValidationFailed(
                fn_def.id().clone(),
                None,
            ))
        }
    }

    fn codegen_variant_body(
        &mut self,
        fn_def: &FnDef,
        variant_index: usize,
        params: &[inkwell::values::BasicValueEnum<'ctx>],
    ) -> CodegenResult<()> {
        let variants: Vec<_> = fn_def.variants_iter().collect();

        if variant_index >= variants.len() {
            return Ok(());
        }

        let current_variant = &variants[variant_index];
        let FnDefVariant(args, body, _return_type) = current_variant;

        if variant_index == variants.len() - 1 {
            let body_block = self
                .context
                .append_basic_block(self.current_fn_value(), "variant_last_body");

            self.store_pattern_args(args, params)?;

            match body.as_ref() {
                FnDefBody::RogatoFn(expr) => {
                    self.builder.build_unconditional_branch(body_block)?;
                    self.builder.position_at_end(body_block);

                    let compiled_val = self.codegen_expr(expr)?;
                    let ret_val = compiled_val.into_basic_value();
                    self.builder.build_return(Some(&ret_val))?;
                }
                _ => return Err(unknown_error("Cannot compile function with NativeFn body!")),
            }

            return Ok(());
        }

        let mut conditions: Vec<(usize, IntValue<'ctx>)> = Vec::new();

        // Track if we've seen any non-Var pattern (which indicates subsequent Var patterns are catch-alls)
        let mut seen_non_var_pattern = false;

        for (i, arg_pattern) in args.iter().enumerate() {
            // Only break on complex patterns that can't be tested
            match arg_pattern.as_ref() {
                Pattern::Number(num) => {
                    seen_non_var_pattern = true;
                    let num_val = val::number_to_f64(num).unwrap_or(0.0);
                    let const_val = self.context.f32_type().const_float(num_val);
                    let cmp = self.builder.build_float_compare(
                        FloatPredicate::OEQ,
                        params[i].into_float_value(),
                        const_val,
                        &format!("cmp_{}", i),
                    )?;
                    conditions.push((i, cmp));
                }
                Pattern::Var(_) | Pattern::Any => {
                    // Only treat as catch-all if we've seen a non-Var pattern before
                    if seen_non_var_pattern {
                        break;
                    }
                }
                _ => {
                    // Other pattern types - can't test, stop here
                    break;
                }
            }
        }

        let variant_body_block = self.context.append_basic_block(
            self.current_fn_value(),
            &format!("variant_{}_body", variant_index),
        );

        if conditions.is_empty() {
            self.builder
                .build_unconditional_branch(variant_body_block)?;

            self.builder.position_at_end(variant_body_block);

            self.store_pattern_args(args, params)?;

            match body.as_ref() {
                FnDefBody::RogatoFn(expr) => {
                    let compiled_val = self.codegen_expr(expr)?;
                    let ret_val = compiled_val.into_basic_value();
                    self.builder.build_return(Some(&ret_val))?;
                }
                _ => return Err(unknown_error("Cannot compile function with NativeFn body!")),
            }

            Ok(())
        } else {
            let next_test_block = self.context.append_basic_block(
                self.current_fn_value(),
                &format!("test_variant_{}", variant_index + 1),
            );

            let last_cond = conditions.last().unwrap();
            self.builder.build_conditional_branch(
                last_cond.1,
                variant_body_block,
                next_test_block,
            )?;

            self.builder.position_at_end(variant_body_block);

            self.store_pattern_args(args, params)?;

            match body.as_ref() {
                FnDefBody::RogatoFn(expr) => {
                    let compiled_val = self.codegen_expr(expr)?;
                    let ret_val = compiled_val.into_basic_value();
                    self.builder.build_return(Some(&ret_val))?;
                }
                _ => return Err(unknown_error("Cannot compile function with NativeFn body!")),
            }

            self.builder.position_at_end(next_test_block);

            self.codegen_variant_body(fn_def, variant_index + 1, params)
        }
    }

    #[allow(dead_code)]
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

        if variant_index >= 1 {
            let variants: Vec<_> = fn_def.variants_iter().collect();
            for (i, arg_name) in variants[variant_index].0.iter().enumerate() {
                if let Pattern::Var(var_id) = &**arg_name {
                    if self.lookup_var(var_id.as_str()).is_none() {
                        let param_type = CompiledType::from_basic_type_enum(params[i].get_type());
                        let llvm_type = param_type.as_basic_type_enum(self.context);
                        let alloca = self.create_entry_block_alloca(llvm_type, var_id.as_str());
                        self.builder.build_store(alloca, params[i])?;
                        self.store_var(var_id.as_str(), alloca, param_type);
                    }
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
                    .or_default()
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

                    let mut all_variants: Vec<FnDefVariant> = Vec::new();
                    for f in def_vec.iter() {
                        let borrowed = f.borrow();
                        for variant in borrowed.variants_iter() {
                            all_variants.push(variant.clone());
                        }
                    }

                    let combined_id = first_fn_def.id().clone();
                    FnDef::new_with_variants(combined_id, FnDefVariants::new(all_variants))
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

    pub fn codegen_fn_call(
        &mut self,
        fn_call: &FnCall,
        span: Option<Span>,
    ) -> CodegenResult<CompiledValue<'ctx>> {
        let id = &fn_call.id;
        let args = &fn_call.args;

        if id.as_str() == "print" || id.as_str() == "println" {
            return self.codegen_println(id.as_str(), args);
        }

        // Try as a module-level function first
        if let Some(function) = self.get_function(id.as_str()) {
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

            let return_type = CompiledType::from_basic_type_enum(value.get_type());
            return Ok(CompiledValue::from_basic_value(value, return_type));
        }

        // Try as a lambda variable (closure indirect call)
        if let Some(info) = self.lambda_info.get(id.as_str()).cloned() {
            return self.codegen_closure_call(id.as_str(), args, &info);
        }

        Err(CodegenError::FnNotDefined(id.clone(), span))
    }

    /// Compiles an indirect call to a lambda/closure stored in a variable.
    /// Loads the closure struct, extracts the function pointer, and calls it.
    fn codegen_closure_call(
        &mut self,
        var_name: &str,
        args: &FnCallArgs,
        info: &LambdaCallInfo,
    ) -> CodegenResult<CompiledValue<'ctx>> {
        let ptr_type = self.context.ptr_type(AddressSpace::default());

        // Get the closure struct pointer from the variable
        let closure_ptr = self
            .lookup_var(var_name)
            .map(|(ptr, _)| *ptr)
            .ok_or_else(|| CodegenError::VarNotFound(var_name.into(), None))?;

        // Build closure struct type to extract fn_ptr
        let mut struct_fields: Vec<BasicTypeEnum<'ctx>> = vec![ptr_type.into()];
        for cap_type in &info.capture_types {
            struct_fields.push(cap_type.as_basic_type_enum(self.context));
        }
        let closure_struct_type = self.context.struct_type(&struct_fields, false);

        // Load fn_ptr from field 0
        let fn_ptr_gep =
            self.builder
                .build_struct_gep(closure_struct_type, closure_ptr, 0, "fn_ptr_gep")?;
        let fn_ptr = self
            .builder
            .build_load(ptr_type, fn_ptr_gep, "fn_ptr")?
            .into_pointer_value();

        // Build call args: env pointer (if captures) + actual args
        let mut call_args: Vec<BasicMetadataValueEnum> = Vec::new();
        if info.capture_count > 0 {
            call_args.push(closure_ptr.into());
        }
        for arg in args.iter() {
            let compiled_val = self.codegen_expr(arg)?;
            call_args.push(compiled_val.into_basic_value().into());
        }

        // Build the function type for the indirect call
        let mut fn_param_types: Vec<BasicMetadataTypeEnum<'ctx>> = Vec::new();
        if info.capture_count > 0 {
            fn_param_types.push(BasicMetadataTypeEnum::PointerType(ptr_type));
        }
        for arg_type in &info.arg_types {
            fn_param_types.push(arg_type.as_metadata_type_enum(self.context));
        }
        let return_llvm_type = info.return_type.as_basic_type_enum(self.context);
        let fn_type = return_llvm_type.fn_type(&fn_param_types, false);

        // Build indirect call through function pointer
        let call_site =
            self.builder
                .build_indirect_call(fn_type, fn_ptr, &call_args, "lambda_call")?;

        let value = call_site
            .try_as_basic_value()
            .basic()
            .ok_or_else(|| unknown_error("Invalid lambda call produced."))?;

        // Return the correct CompiledValue type based on return_type
        match info.return_type {
            CompiledType::Float => Ok(CompiledValue::Float(value.into_float_value())),
            CompiledType::Int32 => Ok(CompiledValue::Int32(value.into_int_value())),
            CompiledType::Int64 => Ok(CompiledValue::Int64(value.into_int_value())),
            CompiledType::Bool => Ok(CompiledValue::Bool(value.into_int_value())),
            CompiledType::String | CompiledType::Lambda => {
                Ok(CompiledValue::String(value.into_pointer_value()))
            }
        }
    }

    fn codegen_println(
        &mut self,
        fn_name: &str,
        args: &FnCallArgs,
    ) -> CodegenResult<CompiledValue<'ctx>> {
        if args.is_empty() {
            return Err(unknown_error(format!(
                "{} takes at least 1 argument",
                fn_name
            )));
        }

        let arg = args.iter().next().unwrap();
        let compiled_val = self.codegen_expr(arg)?;
        let isprintln = fn_name == "println";
        let printf_fn = self
            .printf
            .ok_or_else(|| unknown_error("printf not initialized"))?;

        let i8_type = self.context.i8_type();
        let _ptr_type = self.context.ptr_type(AddressSpace::default());

        let format_str = match &compiled_val {
            CompiledValue::Float(_) => {
                if isprintln {
                    "%f\n\0"
                } else {
                    "%f\0"
                }
            }
            CompiledValue::Int32(_) | CompiledValue::Int64(_) => {
                if isprintln {
                    "%d\n\0"
                } else {
                    "%d\0"
                }
            }
            CompiledValue::String(_) => {
                if isprintln {
                    "%s\n\0"
                } else {
                    "%s\0"
                }
            }
            CompiledValue::Bool(_) => {
                if isprintln {
                    "%s\n\0"
                } else {
                    "%s\0"
                }
            }
            CompiledValue::Lambda(_, _) => {
                if isprintln {
                    "<lambda>\n\0"
                } else {
                    "<lambda>\0"
                }
            }
        };

        let format_bytes = format_str.as_bytes();
        let format_ints: Vec<_> = format_bytes
            .iter()
            .map(|b| i8_type.const_int(*b as u64, false))
            .collect();
        let format_array = i8_type.const_array(&format_ints);

        let format_str_global = self
            .module
            .add_global(format_array.get_type(), None, "fmt_str");
        format_str_global.set_initializer(&format_array);
        let format_str_ptr = format_str_global.as_pointer_value();

        let mut printf_args: Vec<BasicMetadataValueEnum> = vec![format_str_ptr.into()];

        match &compiled_val {
            CompiledValue::Float(fv) => {
                let double = self.builder.build_float_cast(
                    *fv,
                    self.context.f64_type(),
                    "float_to_double",
                )?;
                printf_args.push(double.into());
            }
            CompiledValue::Int32(iv) => printf_args.push((*iv).into()),
            CompiledValue::Int64(iv) => printf_args.push((*iv).into()),
            CompiledValue::String(pv) => printf_args.push((*pv).into()),
            CompiledValue::Bool(bv) => {
                let int_val = self.builder.build_int_cast(*bv, i8_type, "bool_to_i8")?;
                printf_args.push(int_val.into());
            }
            CompiledValue::Lambda(_, _) => {
                // Lambda format string is a literal with no specifiers; no extra args needed
            }
        };

        self.builder
            .build_call(printf_fn, printf_args.as_slice(), "print_call")?;
        Ok(compiled_val)
    }

    pub fn codegen_op_call(
        &mut self,
        id: &Identifier,
        left_expr: &Expression,
        right_expr: &Expression,
        span: Option<Span>,
    ) -> CodegenResult<CompiledValue<'ctx>> {
        let left = self.codegen_expr(left_expr)?;
        let right = self.codegen_expr(right_expr)?;

        match id.as_str() {
            "+" => self.build_numeric_binop(
                &left,
                &right,
                Builder::build_float_add,
                Builder::build_int_add,
                "tmp_add",
                "+",
            ),
            "-" => self.build_numeric_binop(
                &left,
                &right,
                Builder::build_float_sub,
                Builder::build_int_sub,
                "tmp_sub",
                "-",
            ),
            "*" => self.build_numeric_binop(
                &left,
                &right,
                Builder::build_float_mul,
                Builder::build_int_mul,
                "tmp_mul",
                "*",
            ),
            "/" => self.build_numeric_binop(
                &left,
                &right,
                Builder::build_float_div,
                Builder::build_int_signed_div,
                "tmp_div",
                "/",
            ),
            "%" => self.build_numeric_binop(
                &left,
                &right,
                Builder::build_float_rem,
                Builder::build_int_signed_rem,
                "tmp_rem",
                "%",
            ),
            ">" => self.build_comparison_op(
                &left,
                &right,
                FloatPredicate::OGT,
                IntPredicate::SGT,
                "cmp",
            ),
            "<" => self.build_comparison_op(
                &left,
                &right,
                FloatPredicate::OLT,
                IntPredicate::SLT,
                "cmp",
            ),
            ">=" => self.build_comparison_op(
                &left,
                &right,
                FloatPredicate::OGE,
                IntPredicate::SGE,
                "cmp",
            ),
            "<=" => self.build_comparison_op(
                &left,
                &right,
                FloatPredicate::OLE,
                IntPredicate::SLE,
                "cmp",
            ),
            "==" => self.build_comparison_op(
                &left,
                &right,
                FloatPredicate::OEQ,
                IntPredicate::EQ,
                "cmp",
            ),
            "!=" => self.build_comparison_op(
                &left,
                &right,
                FloatPredicate::ONE,
                IntPredicate::NE,
                "cmp",
            ),
            _ => Err(CodegenError::OpNotDefined(id.clone(), span)),
        }
    }

    fn build_numeric_binop(
        &self,
        left: &CompiledValue<'ctx>,
        right: &CompiledValue<'ctx>,
        float_op: impl Fn(
            &Builder<'ctx>,
            FloatValue<'ctx>,
            FloatValue<'ctx>,
            &str,
        ) -> Result<FloatValue<'ctx>, BuilderError>,
        int_op: impl Fn(
            &Builder<'ctx>,
            IntValue<'ctx>,
            IntValue<'ctx>,
            &str,
        ) -> Result<IntValue<'ctx>, BuilderError>,
        name: &str,
        op_symbol: &str,
    ) -> CodegenResult<CompiledValue<'ctx>> {
        match (left, right) {
            (CompiledValue::Float(l), CompiledValue::Float(r)) => {
                Ok(CompiledValue::Float(float_op(self.builder, *l, *r, name)?))
            }
            (CompiledValue::Int32(l), CompiledValue::Int32(r)) => {
                Ok(CompiledValue::Int32(int_op(self.builder, *l, *r, name)?))
            }
            (CompiledValue::Int64(l), CompiledValue::Int64(r)) => {
                Ok(CompiledValue::Int64(int_op(self.builder, *l, *r, name)?))
            }
            _ => Err(unknown_error(format!(
                "Type mismatch for {op_symbol} operation"
            ))),
        }
    }

    fn build_comparison_op(
        &self,
        left: &CompiledValue<'ctx>,
        right: &CompiledValue<'ctx>,
        float_pred: FloatPredicate,
        int_pred: IntPredicate,
        name: &str,
    ) -> CodegenResult<CompiledValue<'ctx>> {
        match (left, right) {
            (CompiledValue::Float(l), CompiledValue::Float(r)) => Ok(CompiledValue::Bool(
                self.builder.build_float_compare(float_pred, *l, *r, name)?,
            )),
            (CompiledValue::Int32(l), CompiledValue::Int32(r)) => Ok(CompiledValue::Bool(
                self.builder.build_int_compare(int_pred, *l, *r, name)?,
            )),
            (CompiledValue::Int64(l), CompiledValue::Int64(r)) => Ok(CompiledValue::Bool(
                self.builder.build_int_compare(int_pred, *l, *r, name)?,
            )),
            _ => Err(unknown_error("Type mismatch for comparison operation")),
        }
    }

    pub fn codegen_module_def(&mut self, _mod_def: &ModuleDef) -> CodegenResult<()> {
        Ok(())
    }

    pub fn codegen_type_def(&mut self, _mod_def: &TypeDef) -> CodegenResult<()> {
        Err(CodegenError::NotYetImplemented("Type definitions".into()))
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
            AST::Use(_id, _imports) => Err(CodegenError::NotYetImplemented(
                "Use/import statements".into(),
            )),
            AST::TypeDef(type_def) => self.codegen_type_def(type_def),
        }
    }

    pub fn codegen_expr(&mut self, expr: &Expression) -> CodegenResult<CompiledValue<'ctx>> {
        match &expr.kind {
            ExprKind::Commented(_c, e) => self.codegen_expr(e),
            ExprKind::Lit(lit_expr) => self.codegen_lit_expr(lit_expr),
            ExprKind::FnCall(fn_call) => self.codegen_fn_call(fn_call, expr.span),
            ExprKind::OpCall(id, left, right) => self.codegen_op_call(id, left, right, expr.span),

            ExprKind::Var(id) => {
                // Check if this variable holds a lambda/closure
                if let Some(info) = self.lambda_info.get(id.as_str()).cloned() {
                    let var_ptr = self
                        .lookup_var(id)
                        .map(|(ptr, _)| *ptr)
                        .ok_or_else(|| CodegenError::VarNotFound(id.into(), expr.span))?;
                    return Ok(CompiledValue::Lambda(var_ptr, info));
                }
                match self.lookup_var(id) {
                    Some((var_ptr, compiled_type)) => {
                        let llvm_type = compiled_type.as_basic_type_enum(self.context);
                        let loaded = self.builder.build_load(llvm_type, *var_ptr, "load_var")?;
                        Ok(CompiledValue::from_basic_value(loaded, *compiled_type))
                    }
                    None => self
                        .codegen_fn_call(&FnCall::new(id.into(), FnCallArgs::empty()), expr.span),
                }
            }

            ExprKind::ConstOrTypeRef(_id) => Err(CodegenError::NotYetImplemented(
                "Constant/type reference expressions".into(),
            )),
            ExprKind::DBTypeRef(_id) => Err(CodegenError::NotYetImplemented(
                "Database type reference expressions".into(),
            )),
            ExprKind::PropFnRef(_id) => Err(CodegenError::NotYetImplemented(
                "Property function reference expressions".into(),
            )),
            ExprKind::EdgeProp(_id, _edge) => Err(CodegenError::NotYetImplemented(
                "Edge property expressions".into(),
            )),
            ExprKind::IfElse(if_else) => self.codegen_if_else(if_else),
            ExprKind::Let(let_expr) => {
                self.push_scope();

                for (var_id, var_expr) in let_expr.bindings.iter() {
                    let compiled_val = self.codegen_expr(var_expr)?;
                    match compiled_val {
                        CompiledValue::Lambda(closure_ptr, ref info) => {
                            // Lambda values: the closure struct pointer IS the variable
                            self.store_var(var_id.as_str(), closure_ptr, CompiledType::Lambda);
                            self.lambda_info.insert(var_id.to_string(), info.clone());
                        }
                        _ => {
                            let val_type = compiled_val.get_type();
                            let llvm_type = val_type.as_basic_type_enum(self.context);
                            let alloca = self.create_entry_block_alloca(llvm_type, var_id.as_str());
                            self.builder
                                .build_store(alloca, compiled_val.into_basic_value())?;
                            self.store_var(var_id.as_str(), alloca, val_type);
                        }
                    }
                }

                let result = self.codegen_expr(&let_expr.body);

                self.pop_scope();

                result
            }
            ExprKind::Lambda(lambda) => self.codegen_lambda(lambda),
            ExprKind::Query(_query) => {
                Err(CodegenError::NotYetImplemented("Query expressions".into()))
            }
            ExprKind::Symbol(_id) => {
                Err(CodegenError::NotYetImplemented("Symbol expressions".into()))
            }
            ExprKind::Quoted(_expr) => {
                Err(CodegenError::NotYetImplemented("Quoted expressions".into()))
            }
            ExprKind::QuotedAST(_ast) => Err(CodegenError::NotYetImplemented(
                "Quoted AST expressions".into(),
            )),
            ExprKind::Unquoted(_expr) => Err(CodegenError::NotYetImplemented(
                "Unquoted expressions".into(),
            )),
            ExprKind::UnquotedAST(_ast) => Err(CodegenError::NotYetImplemented(
                "Unquoted AST expressions".into(),
            )),
            ExprKind::InlineFnDef(fn_def) => {
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

    /// Compiles a lambda expression to an LLVM closure.
    ///
    /// The closure is represented as a struct `{ fn_ptr, captured_var_1, ... }`.
    /// If the lambda captures variables from its enclosing scope, the generated
    /// function takes an extra `env` pointer parameter (the closure struct itself).
    fn codegen_lambda(&mut self, lambda: &Lambda) -> CodegenResult<CompiledValue<'ctx>> {
        let ptr_type = self.context.ptr_type(AddressSpace::default());

        // Get the first variant of the lambda
        let Some(first_variant) = lambda.variants_iter().next() else {
            return Err(unknown_error("Lambda has no variants"));
        };
        let first_variant = first_variant.deref();

        // Analyze free variables (captured from enclosing scope)
        let free_vars = collect_free_vars(lambda);
        let capture_count = free_vars.len();
        let mut sorted_captures: Vec<VarIdentifier> = free_vars.into_iter().collect();
        sorted_captures.sort_by(|a, b| a.as_str().cmp(b.as_str()));

        // Determine argument count and return type
        let arg_count = first_variant.arg_count();
        let return_type = self.infer_expr_type_with_checker(&first_variant.body);
        let return_llvm_type = return_type.as_basic_type_enum(self.context);

        // Build the function type: env pointer (if captures) + lambda args
        let mut fn_arg_types: Vec<BasicMetadataTypeEnum<'ctx>> = Vec::new();
        if capture_count > 0 {
            fn_arg_types.push(BasicMetadataTypeEnum::PointerType(ptr_type));
        }
        let arg_types =
            infer_arg_types_from_patterns(first_variant.args.iter(), &first_variant.body);
        for arg_type in &arg_types {
            fn_arg_types.push(arg_type.as_metadata_type_enum(self.context));
        }

        let fn_type = return_llvm_type.fn_type(&fn_arg_types, false);
        let lambda_name = format!("lambda_{}", self.lambda_counter);
        self.lambda_counter += 1;
        let func = self.module.add_function(&lambda_name, fn_type, None);

        // Determine capture types from the enclosing scope
        let capture_types: Vec<CompiledType> = sorted_captures
            .iter()
            .map(|cap_var| {
                self.lookup_var(cap_var.as_str())
                    .map(|(_, ct)| *ct)
                    .unwrap_or(CompiledType::Float)
            })
            .collect();

        // Build the closure struct type: { fn_ptr, captured_var_1, captured_var_2, ... }
        let mut struct_fields: Vec<BasicTypeEnum<'ctx>> = vec![ptr_type.into()];
        for cap_type in &capture_types {
            struct_fields.push(cap_type.as_basic_type_enum(self.context));
        }
        let closure_struct_type = self.context.struct_type(&struct_fields, false);

        // Compile the lambda function body in an isolated context.
        // with_lambda_context saves and restores: variable scopes, lambda info,
        // current function value, and builder position — even on early return/error.
        let func_result = self.with_lambda_context(|this| -> CodegenResult<()> {
            // Set up the lambda function body
            this.set_current_fn_value(func);
            let entry_block = this.context.append_basic_block(func, "entry");
            this.builder.position_at_end(entry_block);

            // Load captured variables from env struct (if any)
            let param_offset: usize = if capture_count > 0 { 1 } else { 0 };
            if capture_count > 0 {
                let env_ptr = func.get_nth_param(0).unwrap().into_pointer_value();
                for (i, cap_var) in sorted_captures.iter().enumerate() {
                    let cap_type = capture_types[i];
                    let cap_llvm_type = cap_type.as_basic_type_enum(this.context);
                    let gep = this.builder.build_struct_gep(
                        closure_struct_type,
                        env_ptr,
                        (i + 1) as u32,
                        &format!("cap_{}", cap_var.as_str()),
                    )?;
                    let loaded = this.builder.build_load(
                        cap_llvm_type,
                        gep,
                        &format!("load_cap_{}", cap_var.as_str()),
                    )?;
                    let alloca = this.create_entry_block_alloca(cap_llvm_type, cap_var.as_str());
                    this.builder.build_store(alloca, loaded)?;
                    this.store_var(cap_var.as_str(), alloca, cap_type);
                }
            }

            // Store function parameters (lambda args)
            for (i, arg) in func.get_param_iter().skip(param_offset).enumerate() {
                if let Some(pattern) = first_variant.args.get(i) {
                    if let Pattern::Var(var_id) = pattern.deref() {
                        let param_type = CompiledType::from_basic_type_enum(arg.get_type());
                        let llvm_type = param_type.as_basic_type_enum(this.context);
                        let alloca = this.create_entry_block_alloca(llvm_type, var_id.as_str());
                        this.builder.build_store(alloca, arg)?;
                        this.store_var(var_id.as_str(), alloca, param_type);
                    }
                }
            }

            // Compile the lambda body
            let compiled_val = this.codegen_expr(&first_variant.body)?;
            let ret_val = compiled_val.into_basic_value();
            this.builder.build_return(Some(&ret_val))?;

            // Verify and apply passes
            if func.verify(true) {
                this.run_function_passes();
            }

            Ok(())
        });
        func_result?;

        // Allocate and populate the closure struct in the calling function
        let closure_alloc = self.create_entry_block_alloca(closure_struct_type, "closure");

        // Store fn_ptr at field 0
        let fn_ptr = func.as_global_value().as_pointer_value();
        let fn_ptr_gep =
            self.builder
                .build_struct_gep(closure_struct_type, closure_alloc, 0, "fn_ptr_field")?;
        self.builder.build_store(fn_ptr_gep, fn_ptr)?;

        // Store captured variable values from the enclosing scope
        for (i, cap_var) in sorted_captures.iter().enumerate() {
            let cap_gep = self.builder.build_struct_gep(
                closure_struct_type,
                closure_alloc,
                (i + 1) as u32,
                &format!("cap_field_{}", cap_var.as_str()),
            )?;
            let (var_ptr, cap_type) = self
                .lookup_var(cap_var.as_str())
                .copied()
                .ok_or_else(|| CodegenError::VarNotFound(cap_var.as_str().into(), None))?;
            let cap_llvm_type = cap_type.as_basic_type_enum(self.context);
            let var_val = self.builder.build_load(
                cap_llvm_type,
                var_ptr,
                &format!("load_{}", cap_var.as_str()),
            )?;
            self.builder.build_store(cap_gep, var_val)?;
        }

        let call_info = LambdaCallInfo {
            arg_count,
            capture_count,
            capture_types,
            arg_types,
            return_type,
        };

        Ok(CompiledValue::Lambda(closure_alloc, call_info))
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

    fn lookup_var<S: ToString>(&self, name: S) -> Option<&(PointerValue<'ctx>, CompiledType)> {
        let name = name.to_string();
        for scope in self.variable_scopes.iter().rev() {
            if let Some(entry) = scope.get(&name) {
                return Some(entry);
            }
        }
        None
    }

    fn store_var<S: ToString>(
        &mut self,
        name: S,
        pointer_val: PointerValue<'ctx>,
        compiled_type: CompiledType,
    ) {
        if let Some(scope) = self.variable_scopes.last_mut() {
            scope.insert(name.to_string(), (pointer_val, compiled_type));
        }
    }

    fn push_scope(&mut self) {
        self.variable_scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.variable_scopes.pop();
    }

    /// Runs a closure with a completely isolated variable scope stack.
    /// The current scopes are saved and restored after the closure completes.
    /// This is used for compilation contexts where the body should not
    /// see the enclosing function's variables (captures are loaded from the closure struct).
    #[allow(dead_code)]
    fn with_isolated_scope<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let saved_scopes = std::mem::replace(&mut self.variable_scopes, vec![HashMap::new()]);
        let saved_lambda_info = self.lambda_info.clone();
        let result = f(self);
        self.variable_scopes = saved_scopes;
        self.lambda_info = saved_lambda_info;
        result
    }

    /// Runs a closure with full lambda compilation context isolation.
    /// Saves and restores: variable scopes, lambda info, current function value,
    /// and builder insertion position. This ensures that lambda body compilation
    /// cannot corrupt the enclosing compilation state, even on early returns or errors.
    fn with_lambda_context<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let saved_fn = self.current_fn_value;
        let saved_block = self.builder.get_insert_block();
        let saved_scopes = std::mem::replace(&mut self.variable_scopes, vec![HashMap::new()]);
        let saved_lambda_info = self.lambda_info.clone();
        let result = f(self);
        self.variable_scopes = saved_scopes;
        self.lambda_info = saved_lambda_info;
        self.current_fn_value = saved_fn;
        if let Some(block) = saved_block {
            self.builder.position_at_end(block);
        }
        result
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
