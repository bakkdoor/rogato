use std::{collections::HashMap, fmt, ops::Deref, rc::Rc};

use rogato_common::ast::{
    expression::{Expression, Literal},
    fn_call::FnCall,
    fn_def::FnDef,
    if_else::IfElse,
    lambda::LambdaArgs,
    let_expression::LetExpression,
    literal::TupleItems,
    query::{Query, QueryBinding},
    type_expression::TypeExpression,
    Identifier, VarIdentifier,
};

#[cfg(test)]
pub mod tests;

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum TypeCheckError {
    #[error("Undefined variable: {0}")]
    UndefinedVariable(VarIdentifier),

    #[error("Undefined function: {0}")]
    UndefinedFunction(Identifier),

    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch {
        expected: Rc<TypeExpression>,
        actual: Rc<TypeExpression>,
    },

    #[error("Argument count mismatch: expected {expected} arguments, got {actual}")]
    ArgumentCountMismatch { expected: usize, actual: usize },

    #[error("Type error at argument {position}: {message}")]
    ArgumentError { position: usize, message: String },

    #[error("Cannot infer type for expression")]
    CannotInferType,

    #[error("Unknown type checking error: {0}")]
    Unknown(String),
}

#[derive(Clone)]
pub struct TypeEnvironment {
    variables: HashMap<VarIdentifier, Rc<TypeExpression>>,
    functions: HashMap<Identifier, FnSignature>,
    type_defs: HashMap<Identifier, Rc<TypeExpression>>,
}

#[derive(Clone, Debug)]
pub struct FnSignature {
    pub name: Identifier,
    pub arg_types: Vec<Rc<TypeExpression>>,
    pub return_type: Rc<TypeExpression>,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        TypeEnvironment {
            variables: HashMap::new(),
            functions: HashMap::new(),
            type_defs: HashMap::new(),
        }
    }

    pub fn insert_variable(&mut self, id: VarIdentifier, type_expr: Rc<TypeExpression>) {
        self.variables.insert(id, type_expr);
    }

    pub fn lookup_variable(&self, id: &VarIdentifier) -> Option<&Rc<TypeExpression>> {
        self.variables.get(id)
    }

    pub fn insert_function(&mut self, sig: FnSignature) {
        self.functions.insert(sig.name.clone(), sig);
    }

    pub fn lookup_function(&self, name: &Identifier) -> Option<&FnSignature> {
        self.functions.get(name)
    }

    pub fn insert_type_def(&mut self, id: Identifier, type_expr: Rc<TypeExpression>) {
        self.type_defs.insert(id, type_expr);
    }

    pub fn lookup_type_def(&self, id: &Identifier) -> Option<&Rc<TypeExpression>> {
        self.type_defs.get(id)
    }

    pub fn new_scope(&self) -> TypeEnvironment {
        let mut new_env = self.clone();
        new_env.variables = self.variables.clone();
        new_env.functions = self.functions.clone();
        new_env.type_defs = self.type_defs.clone();
        new_env
    }
}

impl Default for TypeEnvironment {
    fn default() -> Self {
        TypeEnvironment::new()
    }
}

impl fmt::Debug for TypeEnvironment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypeEnvironment")
            .field("variables", &self.variables.keys().collect::<Vec<_>>())
            .field("functions", &self.functions.keys().collect::<Vec<_>>())
            .finish()
    }
}

#[derive(Clone, Debug)]
pub enum InferredType {
    Known(Rc<TypeExpression>),
    Unknown,
}

impl InferredType {
    pub fn known(type_expr: Rc<TypeExpression>) -> Self {
        InferredType::Known(type_expr)
    }

    pub fn unknown() -> Self {
        InferredType::Unknown
    }

    pub fn is_known(&self) -> bool {
        matches!(self, InferredType::Known(_))
    }

    pub fn inner(&self) -> Option<&Rc<TypeExpression>> {
        match self {
            InferredType::Known(te) => Some(te),
            InferredType::Unknown => None,
        }
    }

    pub fn unify(&self, other: &InferredType) -> InferredType {
        match (self, other) {
            (InferredType::Known(a), InferredType::Known(b)) if a == b => self.clone(),
            (InferredType::Unknown, _) => other.clone(),
            (_, InferredType::Unknown) => self.clone(),
            _ => InferredType::Unknown,
        }
    }
}

pub struct TypeInferrer {
    env: TypeEnvironment,
}

impl TypeInferrer {
    pub fn new() -> Self {
        let mut env = TypeEnvironment::new();
        insert_builtin_types(&mut env);
        TypeInferrer { env }
    }

    pub fn with_env(env: TypeEnvironment) -> Self {
        TypeInferrer { env }
    }

    pub fn env(&self) -> &TypeEnvironment {
        &self.env
    }

    pub fn env_mut(&mut self) -> &mut TypeEnvironment {
        &mut self.env
    }

    pub fn infer_expression(&self, expr: &Expression) -> InferredType {
        match expr {
            Expression::Lit(lit) => self.infer_literal(lit),
            Expression::Var(id) => match self.env.lookup_variable(id) {
                Some(te) => InferredType::Known(Rc::clone(te)),
                None => InferredType::Unknown,
            },
            Expression::FnCall(fn_call) => {
                if let Some(sig) = self.env.lookup_function(&fn_call.id) {
                    InferredType::Known(Rc::clone(&sig.return_type))
                } else {
                    InferredType::Unknown
                }
            }
            Expression::OpCall(op, _left, _right) => self.infer_op_call(op),
            Expression::IfElse(if_else) => {
                let then_type = self.infer_expression(&if_else.then_expr);
                let else_type = self.infer_expression(&if_else.else_expr);
                then_type.unify(&else_type)
            }
            Expression::Let(let_expr) => self.infer_expression(&let_expr.body),
            Expression::Lambda(lambda) => {
                let lambda = lambda.deref();
                if let Some(first_variant) = lambda.variants_iter().next() {
                    let first_variant = first_variant.deref();
                    let arg_types: Vec<TypeExpression> = (0..first_variant.arg_count())
                        .map(|_| TypeExpression::Unknown)
                        .collect();
                    let return_type = self.infer_expression(&first_variant.body);
                    InferredType::Known(Rc::new(TypeExpression::FunctionType(
                        LambdaArgs::new(arg_types),
                        return_type
                            .inner()
                            .cloned()
                            .unwrap_or_else(|| Rc::new(TypeExpression::Unknown)),
                    )))
                } else {
                    InferredType::Unknown
                }
            }
            Expression::ConstOrTypeRef(id) => {
                if let Some(te) = self.env.lookup_type_def(id) {
                    InferredType::Known(Rc::clone(te))
                } else {
                    InferredType::Unknown
                }
            }
            Expression::DBTypeRef(_) => InferredType::Known(Rc::new(TypeExpression::SymbolType)),
            Expression::PropFnRef(_) => InferredType::Unknown,
            Expression::EdgeProp(_, _) => InferredType::Known(Rc::new(TypeExpression::SymbolType)),
            Expression::Query(_) => InferredType::Unknown,
            Expression::Symbol(_id) => InferredType::Known(Rc::new(TypeExpression::SymbolType)),
            Expression::Quoted(_) => InferredType::Unknown,
            Expression::QuotedAST(_) => InferredType::Unknown,
            Expression::Unquoted(expr) => self.infer_expression(expr),
            Expression::UnquotedAST(_) => InferredType::Unknown,
            Expression::InlineFnDef(fn_def) => self.infer_fn_def(&fn_def.borrow()),
            Expression::Commented(_, expr) => self.infer_expression(expr),
        }
    }

    fn infer_literal(&self, lit: &Literal) -> InferredType {
        match lit {
            Literal::Number(_) => InferredType::Known(Rc::new(TypeExpression::NumberType)),
            Literal::String(_) => InferredType::Known(Rc::new(TypeExpression::StringType)),
            Literal::Bool(_) => InferredType::Known(Rc::new(TypeExpression::BoolType)),
            Literal::Tuple(items) => {
                let types: Vec<Rc<TypeExpression>> = items
                    .iter()
                    .map(|item| match self.infer_expression(item) {
                        InferredType::Known(te) => te,
                        _ => Rc::new(TypeExpression::Unknown),
                    })
                    .collect();
                InferredType::Known(Rc::new(TypeExpression::TupleType(TupleItems::from(types))))
            }
            Literal::List(items) => {
                if items.is_empty() {
                    return InferredType::Known(Rc::new(TypeExpression::ListType(Rc::new(
                        TypeExpression::Unknown,
                    ))));
                }
                let mut element_type: Option<Rc<TypeExpression>> = None;
                for item in items.iter() {
                    let inferred = self.infer_expression(item);
                    if let InferredType::Known(te) = inferred {
                        element_type = Some(match &element_type {
                            None => te,
                            Some(_existing) => Rc::new(TypeExpression::Unknown),
                        });
                    }
                }
                InferredType::Known(Rc::new(TypeExpression::ListType(
                    element_type.unwrap_or_else(|| Rc::new(TypeExpression::Unknown)),
                )))
            }
            Literal::ListCons(_, _) => InferredType::Unknown,
            Literal::Struct(_, _) => InferredType::Unknown,
            Literal::Map(kv_pairs) => {
                let mut key_type: Option<Rc<TypeExpression>> = None;
                let mut val_type: Option<Rc<TypeExpression>> = None;
                for kv_pair in kv_pairs.iter() {
                    let k_inferred = self.infer_expression(&kv_pair.key);
                    let v_inferred = self.infer_expression(&kv_pair.value);
                    if let InferredType::Known(kt) = k_inferred {
                        key_type = Some(kt);
                    }
                    if let InferredType::Known(vt) = v_inferred {
                        val_type = Some(vt);
                    }
                }
                InferredType::Known(Rc::new(TypeExpression::MapType(
                    key_type.unwrap_or_else(|| Rc::new(TypeExpression::Unknown)),
                    val_type.unwrap_or_else(|| Rc::new(TypeExpression::Unknown)),
                )))
            }
            Literal::MapCons(_, _) => InferredType::Unknown,
        }
    }

    fn infer_op_call(&self, op: &Identifier) -> InferredType {
        match op.as_str() {
            "+" | "-" | "*" | "/" | "%" => InferredType::Known(Rc::new(TypeExpression::NumberType)),
            ">" | "<" | ">=" | "<=" => InferredType::Known(Rc::new(TypeExpression::NumberType)),
            "==" | "!=" => InferredType::Known(Rc::new(TypeExpression::BoolType)),
            "&&" | "||" => InferredType::Known(Rc::new(TypeExpression::BoolType)),
            _ => InferredType::Unknown,
        }
    }

    pub fn infer_fn_def(&self, fn_def: &FnDef) -> InferredType {
        let mut inferrer = TypeInferrer::with_env(self.env.clone());

        for variant in fn_def.variants_iter() {
            let args = variant.0.iter();
            let arg_types: Vec<Rc<TypeExpression>> =
                args.map(|_| Rc::new(TypeExpression::Unknown)).collect();

            let return_type = if let Some(rt) = variant.return_type() {
                Rc::clone(rt)
            } else {
                match &*variant.1 {
                    rogato_common::ast::fn_def::FnDefBody::RogatoFn(body) => {
                        match inferrer.infer_expression(body) {
                            InferredType::Known(te) => te,
                            _ => Rc::new(TypeExpression::Unknown),
                        }
                    }
                    rogato_common::ast::fn_def::FnDefBody::NativeFn(_) => {
                        Rc::new(TypeExpression::Unknown)
                    }
                }
            };

            let sig = FnSignature {
                name: fn_def.id().clone(),
                arg_types,
                return_type,
            };
            inferrer.env_mut().insert_function(sig);
        }

        InferredType::Unknown
    }
}

impl Default for TypeInferrer {
    fn default() -> Self {
        TypeInferrer::new()
    }
}

fn insert_builtin_types(env: &mut TypeEnvironment) {
    env.insert_type_def("Number".into(), Rc::new(TypeExpression::NumberType));
    env.insert_type_def("String".into(), Rc::new(TypeExpression::StringType));
    env.insert_type_def("Bool".into(), Rc::new(TypeExpression::BoolType));
    env.insert_type_def("Symbol".into(), Rc::new(TypeExpression::SymbolType));
}

pub trait TypeCheck<T> {
    fn type_check(&self, context: &mut TypeEnvironment) -> Result<T, TypeCheckError>;
}

impl TypeCheck<InferredType> for Expression {
    fn type_check(&self, context: &mut TypeEnvironment) -> Result<InferredType, TypeCheckError> {
        match self {
            Expression::Lit(lit) => Ok(type_check_literal(lit)),
            Expression::Var(id) => match context.lookup_variable(id) {
                Some(te) => Ok(InferredType::Known(Rc::clone(te))),
                None => Err(TypeCheckError::UndefinedVariable(id.clone())),
            },
            Expression::FnCall(fn_call) => type_check_fn_call(fn_call, context),
            Expression::OpCall(op, left, right) => type_check_op_call(op, left, right, context),
            Expression::IfElse(if_else) => type_check_if_else(if_else, context),
            Expression::Let(let_expr) => type_check_let_expression(let_expr, context),
            Expression::Lambda(lambda) => {
                let lambda = lambda.deref();
                if let Some(first_variant) = lambda.variants_iter().next() {
                    let first_variant = first_variant.deref();
                    let arg_types: Vec<TypeExpression> = (0..first_variant.arg_count())
                        .map(|_| TypeExpression::Unknown)
                        .collect();
                    let return_type = first_variant.body.type_check(context)?;
                    Ok(InferredType::Known(Rc::new(TypeExpression::FunctionType(
                        LambdaArgs::new(arg_types),
                        return_type
                            .inner()
                            .cloned()
                            .unwrap_or_else(|| Rc::new(TypeExpression::Unknown)),
                    ))))
                } else {
                    Ok(InferredType::Unknown)
                }
            }
            Expression::Query(query) => type_check_query(query, context),
            Expression::ConstOrTypeRef(id) => {
                if let Some(te) = context.lookup_type_def(id) {
                    Ok(InferredType::Known(Rc::clone(te)))
                } else {
                    Err(TypeCheckError::UndefinedVariable(id.clone().into()))
                }
            }
            Expression::DBTypeRef(_) => {
                Ok(InferredType::Known(Rc::new(TypeExpression::SymbolType)))
            }
            Expression::PropFnRef(_) => Ok(InferredType::Unknown),
            Expression::EdgeProp(_, _) => {
                Ok(InferredType::Known(Rc::new(TypeExpression::SymbolType)))
            }
            Expression::Symbol(_) => Ok(InferredType::Known(Rc::new(TypeExpression::SymbolType))),
            Expression::Quoted(_) => Ok(InferredType::Unknown),
            Expression::QuotedAST(_) => Ok(InferredType::Unknown),
            Expression::Unquoted(expr) => expr.type_check(context),
            Expression::UnquotedAST(_) => Ok(InferredType::Unknown),
            Expression::InlineFnDef(fn_def) => fn_def.borrow().type_check(context),
            Expression::Commented(_, expr) => expr.type_check(context),
        }
    }
}

fn type_check_literal(lit: &Literal) -> InferredType {
    match lit {
        Literal::Number(_) => InferredType::Known(Rc::new(TypeExpression::NumberType)),
        Literal::String(_) => InferredType::Known(Rc::new(TypeExpression::StringType)),
        Literal::Bool(_) => InferredType::Known(Rc::new(TypeExpression::BoolType)),
        Literal::Tuple(items) => {
            let types: Vec<Rc<TypeExpression>> = items
                .iter()
                .map(|item| {
                    if let Ok(InferredType::Known(te)) =
                        item.type_check(&mut TypeEnvironment::new())
                    {
                        te
                    } else {
                        Rc::new(TypeExpression::Unknown)
                    }
                })
                .collect();
            InferredType::Known(Rc::new(TypeExpression::TupleType(TupleItems::from(types))))
        }
        Literal::List(items) => {
            if items.is_empty() {
                return InferredType::Known(Rc::new(TypeExpression::ListType(Rc::new(
                    TypeExpression::Unknown,
                ))));
            }
            let mut element_type: Option<Rc<TypeExpression>> = None;
            for item in items.iter() {
                if let Ok(InferredType::Known(te)) = item.type_check(&mut TypeEnvironment::new()) {
                    element_type = Some(match &element_type {
                        None => te,
                        Some(_) => Rc::new(TypeExpression::Unknown),
                    });
                }
            }
            InferredType::Known(Rc::new(TypeExpression::ListType(
                element_type.unwrap_or_else(|| Rc::new(TypeExpression::Unknown)),
            )))
        }
        Literal::ListCons(_, _) => InferredType::Unknown,
        Literal::Struct(_, _) => InferredType::Unknown,
        Literal::Map(kv_pairs) => {
            let mut key_type: Option<Rc<TypeExpression>> = None;
            let mut val_type: Option<Rc<TypeExpression>> = None;
            for kv_pair in kv_pairs.iter() {
                if let Ok(InferredType::Known(kt)) =
                    kv_pair.key.type_check(&mut TypeEnvironment::new())
                {
                    key_type = Some(kt);
                }
                if let Ok(InferredType::Known(vt)) =
                    kv_pair.value.type_check(&mut TypeEnvironment::new())
                {
                    val_type = Some(vt);
                }
            }
            InferredType::Known(Rc::new(TypeExpression::MapType(
                key_type.unwrap_or_else(|| Rc::new(TypeExpression::Unknown)),
                val_type.unwrap_or_else(|| Rc::new(TypeExpression::Unknown)),
            )))
        }
        Literal::MapCons(_, _) => InferredType::Unknown,
    }
}

fn type_check_fn_call(
    fn_call: &FnCall,
    context: &mut TypeEnvironment,
) -> Result<InferredType, TypeCheckError> {
    let (arg_types, return_type) = match context.lookup_function(&fn_call.id) {
        Some(sig) => (sig.arg_types.clone(), sig.return_type.clone()),
        None => return Err(TypeCheckError::UndefinedFunction(fn_call.id.clone())),
    };

    if arg_types.len() != fn_call.args.len() {
        return Err(TypeCheckError::ArgumentCountMismatch {
            expected: arg_types.len(),
            actual: fn_call.args.len(),
        });
    }

    for arg in fn_call.args.iter() {
        let _arg_type = arg.type_check(context)?;
    }

    Ok(InferredType::Known(Rc::clone(&return_type)))
}

fn type_check_op_call(
    op: &Identifier,
    left: &Rc<Expression>,
    right: &Rc<Expression>,
    context: &mut TypeEnvironment,
) -> Result<InferredType, TypeCheckError> {
    let left_type = left.type_check(context)?;
    let right_type = right.type_check(context)?;

    match op.as_str() {
        "+" | "-" | "*" | "/" | "%" => {
            ensure_type(&left_type, &TypeExpression::NumberType)?;
            ensure_type(&right_type, &TypeExpression::NumberType)?;
            Ok(InferredType::Known(Rc::new(TypeExpression::NumberType)))
        }
        ">" | "<" | ">=" | "<=" => {
            ensure_type(&left_type, &TypeExpression::NumberType)?;
            ensure_type(&right_type, &TypeExpression::NumberType)?;
            Ok(InferredType::Known(Rc::new(TypeExpression::BoolType)))
        }
        "==" | "!=" => Ok(InferredType::Known(Rc::new(TypeExpression::BoolType))),
        "&&" | "||" => {
            ensure_type(&left_type, &TypeExpression::BoolType)?;
            ensure_type(&right_type, &TypeExpression::BoolType)?;
            Ok(InferredType::Known(Rc::new(TypeExpression::BoolType)))
        }
        _ => Ok(InferredType::Unknown),
    }
}

fn ensure_type(inferred: &InferredType, expected: &TypeExpression) -> Result<(), TypeCheckError> {
    if let InferredType::Known(actual) = inferred {
        if actual.as_ref() != expected {
            return Err(TypeCheckError::TypeMismatch {
                expected: Rc::new(expected.clone()),
                actual: Rc::clone(actual),
            });
        }
    }
    Ok(())
}

fn type_check_if_else(
    if_else: &IfElse,
    context: &mut TypeEnvironment,
) -> Result<InferredType, TypeCheckError> {
    let cond_type = if_else.condition.type_check(context)?;
    ensure_type(&cond_type, &TypeExpression::BoolType)?;

    let then_type = if_else.then_expr.type_check(context)?;
    let else_type = if_else.else_expr.type_check(context)?;

    Ok(then_type.unify(&else_type))
}

fn type_check_let_expression(
    let_expr: &LetExpression,
    context: &mut TypeEnvironment,
) -> Result<InferredType, TypeCheckError> {
    let mut new_scope = context.new_scope();

    for (id, val) in let_expr.bindings.iter() {
        let binding_type = val.type_check(context)?;
        if let InferredType::Known(te) = binding_type {
            new_scope.insert_variable(id.clone(), te);
        }
    }

    let mut final_context = context.new_scope();
    for (id, _) in let_expr.bindings.iter() {
        if let Some(te) = new_scope.lookup_variable(id) {
            final_context.insert_variable(id.clone(), Rc::clone(te));
        }
    }

    let_expr.body.type_check(&mut final_context)
}

fn type_check_query(
    query: &Query,
    context: &mut TypeEnvironment,
) -> Result<InferredType, TypeCheckError> {
    for binding in query.bindings().iter() {
        type_check_query_binding(binding, context)?;
    }

    for guard in query.guards().iter() {
        let guard_type = guard.type_check(context)?;
        ensure_type(&guard_type, &TypeExpression::BoolType)?;
    }

    let _production_type = query.production().type_check(context)?;

    Ok(InferredType::Known(Rc::new(TypeExpression::SymbolType)))
}

fn type_check_query_binding(
    binding: &QueryBinding,
    context: &mut TypeEnvironment,
) -> Result<(), TypeCheckError> {
    let source_type = binding.val().type_check(context)?;

    for id in binding.ids().iter() {
        if let InferredType::Known(te) = &source_type {
            context.insert_variable(id.clone(), Rc::clone(te));
        }
    }

    Ok(())
}

impl TypeCheck<InferredType> for FnDef {
    fn type_check(&self, context: &mut TypeEnvironment) -> Result<InferredType, TypeCheckError> {
        let mut inferrer = TypeInferrer::with_env(context.clone());

        for variant in self.variants_iter() {
            let args = variant.0.iter();
            let arg_types: Vec<Rc<TypeExpression>> =
                args.map(|_| Rc::new(TypeExpression::Unknown)).collect();

            let return_type = if let Some(rt) = variant.return_type() {
                Rc::clone(rt)
            } else {
                match &*variant.1 {
                    rogato_common::ast::fn_def::FnDefBody::RogatoFn(body) => {
                        match inferrer.infer_expression(body) {
                            InferredType::Known(te) => te,
                            _ => Rc::new(TypeExpression::Unknown),
                        }
                    }
                    rogato_common::ast::fn_def::FnDefBody::NativeFn(_) => {
                        Rc::new(TypeExpression::Unknown)
                    }
                }
            };

            let sig = FnSignature {
                name: self.id().clone(),
                arg_types,
                return_type,
            };
            inferrer.env_mut().insert_function(sig);
        }

        Ok(InferredType::Unknown)
    }
}
