use std::{ops::Deref, rc::Rc};

use rogato_common::ast::{
    expression::{Expression, Literal},
    fn_def::FnDef,
    lambda::LambdaArgs,
    literal::TupleItems,
    type_expression::TypeExpression,
    Identifier,
};

use crate::environment::{FnSignature, TypeEnvironment};
use crate::inferred_type::InferredType;
use crate::pattern::infer_pattern_type;

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
                    let mut child_env = self.env.new_scope();

                    let mut arg_types = Vec::new();
                    for arg_pattern in first_variant.args.iter() {
                        let (arg_type, var_bindings) = infer_pattern_type(arg_pattern);
                        arg_types.push(arg_type);
                        for (var_id, var_type) in var_bindings {
                            child_env.insert_variable(var_id, Rc::new(var_type));
                        }
                    }

                    let child_inferrer = TypeInferrer::with_env(child_env);
                    let return_type = child_inferrer.infer_expression(&first_variant.body);

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
            ">" | "<" | ">=" | "<=" => InferredType::Known(Rc::new(TypeExpression::BoolType)),
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
