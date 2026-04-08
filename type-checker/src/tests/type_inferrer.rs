use rogato_common::ast::{
    expression::{ExprKind, Expression},
    fn_call::{FnCall, FnCallArgs},
    if_else::IfElse,
    lambda::{Lambda, LambdaArgs, LambdaVariant},
    let_expression::{LetBindings, LetExpression},
    literal::Literal,
    pattern::Pattern,
    query::{Query, QueryBinding, QueryBindings, QueryGuards},
    type_expression::TypeExpression,
    Identifier, VarIdentifier,
};
use std::ops::Deref;
use std::rc::Rc;

use crate::{InferredType, TypeCheck, TypeEnvironment, TypeInferrer};

#[test]
fn infer_number_literal() {
    let inferrer = TypeInferrer::new();
    let literal = Literal::Number(42i32.into());
    let expr = Expression::unspanned(ExprKind::Lit(literal));

    let result = inferrer.infer_expression(&expr);
    assert!(matches!(result, InferredType::Known(_)));
}

#[test]
fn infer_string_literal() {
    let inferrer = TypeInferrer::new();
    let literal = Literal::String("hello".to_string());
    let expr = Expression::unspanned(ExprKind::Lit(literal));

    let result = inferrer.infer_expression(&expr);
    assert!(matches!(result, InferredType::Known(_)));
}

#[test]
fn infer_bool_literal() {
    let inferrer = TypeInferrer::new();
    let literal = Literal::Bool(true);
    let expr = Expression::unspanned(ExprKind::Lit(literal));

    let result = inferrer.infer_expression(&expr);
    assert!(matches!(result, InferredType::Known(_)));
}

#[test]
fn infer_addition_op() {
    let inferrer = TypeInferrer::new();

    use rogato_common::ast::literal::Literal;
    let left = Literal::Number(1i32.into());
    let right = Literal::Number(2i32.into());

    let op = "+".into();
    let left_expr = Expression::rc(ExprKind::Lit(left));
    let right_expr = Expression::rc(ExprKind::Lit(right));
    let expr = Expression::unspanned(ExprKind::OpCall(op, left_expr, right_expr));

    let result = inferrer.infer_expression(&expr);
    assert!(matches!(result, InferredType::Known(_)));
}

#[test]
fn infer_undefined_variable() {
    let inferrer = TypeInferrer::new();

    let var: VarIdentifier = "undefined_var".into();
    let expr = Expression::unspanned(ExprKind::Var(var));

    let result = inferrer.infer_expression(&expr);
    assert!(matches!(result, InferredType::Unknown));
}

#[test]
fn infer_defined_variable() {
    let mut inferrer = TypeInferrer::new();

    inferrer
        .env_mut()
        .insert_variable("x".into(), Rc::new(TypeExpression::NumberType));

    let var: VarIdentifier = "x".into();
    let expr = Expression::unspanned(ExprKind::Var(var));

    let result = inferrer.infer_expression(&expr);
    assert!(matches!(result, InferredType::Known(_)));
}

#[test]
fn infer_list_literal() {
    let inferrer = TypeInferrer::new();

    let items: Vec<Rc<Expression>> = vec![
        Expression::rc(ExprKind::Lit(Literal::Number(1i32.into()))),
        Expression::rc(ExprKind::Lit(Literal::Number(2i32.into()))),
    ];

    let literal = Literal::List(rogato_common::ast::literal::TupleItems::from(items));
    let expr = Expression::unspanned(ExprKind::Lit(literal));

    let result = inferrer.infer_expression(&expr);
    assert!(matches!(result, InferredType::Known(_)));
}

#[test]
fn infer_tuple_literal() {
    let inferrer = TypeInferrer::new();

    let items: Vec<Rc<Expression>> = vec![
        Expression::rc(ExprKind::Lit(Literal::Number(1i32.into()))),
        Expression::rc(ExprKind::Lit(Literal::String("hello".to_string()))),
    ];

    let literal = Literal::Tuple(rogato_common::ast::literal::TupleItems::from(items));
    let expr = Expression::unspanned(ExprKind::Lit(literal));

    let result = inferrer.infer_expression(&expr);
    assert!(matches!(result, InferredType::Known(_)));
}

#[test]
fn infer_comparison_op() {
    let inferrer = TypeInferrer::new();

    let left = Literal::Number(1i32.into());
    let right = Literal::Number(2i32.into());

    let op: Identifier = ">".into();
    let left_expr = Expression::rc(ExprKind::Lit(left));
    let right_expr = Expression::rc(ExprKind::Lit(right));
    let expr = Expression::unspanned(ExprKind::OpCall(op, left_expr, right_expr));

    let result = inferrer.infer_expression(&expr);
    assert!(matches!(result, InferredType::Known(_)));
}

#[test]
fn infer_logical_op() {
    let inferrer = TypeInferrer::new();

    let left = Literal::Bool(true);
    let right = Literal::Bool(false);

    let op: Identifier = "&&".into();
    let left_expr = Expression::rc(ExprKind::Lit(left));
    let right_expr = Expression::rc(ExprKind::Lit(right));
    let expr = Expression::unspanned(ExprKind::OpCall(op, left_expr, right_expr));

    let result = inferrer.infer_expression(&expr);
    assert!(matches!(result, InferredType::Known(_)));
}

#[cfg(test)]
mod type_check_tests {
    use super::*;

    #[test]
    fn type_check_undefined_variable_returns_error() {
        let var: VarIdentifier = "undefined_var".into();
        let expr = Expression::unspanned(ExprKind::Var(var));
        let mut context = TypeEnvironment::new();

        let result = expr.type_check(&mut context);
        assert!(result.is_err());
    }

    #[test]
    fn type_check_defined_variable_returns_type() {
        let var: VarIdentifier = "x".into();
        let expr = Expression::unspanned(ExprKind::Var(var));
        let mut context = TypeEnvironment::new();

        context.insert_variable("x".into(), Rc::new(TypeExpression::NumberType));

        let result = expr.type_check(&mut context);
        assert!(matches!(result, Ok(InferredType::Known(_))));
    }

    #[test]
    fn type_check_undefined_function_returns_error() {
        let args = FnCallArgs::new(vec![]);
        let fn_call = FnCall::new("undefined_fn".into(), args);
        let expr = Expression::unspanned(ExprKind::FnCall(fn_call));
        let mut context = TypeEnvironment::new();

        let result = expr.type_check(&mut context);
        assert!(matches!(
            result,
            Err(crate::TypeCheckError::UndefinedFunction(_, _))
        ));
    }

    #[test]
    fn type_check_function_call_returns_return_type() {
        let args = FnCallArgs::new(vec![]);
        let fn_call = FnCall::new("my_fn".into(), args);
        let expr = Expression::unspanned(ExprKind::FnCall(fn_call));
        let mut context = TypeEnvironment::new();

        context.insert_function(crate::FnSignature {
            name: "my_fn".into(),
            arg_types: vec![],
            return_type: Rc::new(TypeExpression::NumberType),
        });

        let result = expr.type_check(&mut context);
        assert!(matches!(result, Ok(InferredType::Known(_))));
    }

    #[test]
    fn type_check_fn_call_argument_count_mismatch() {
        let args = FnCallArgs::new(vec![Expression::rc(ExprKind::Lit(Literal::Number(
            1i32.into(),
        )))]);
        let fn_call = FnCall::new("my_fn".into(), args);
        let expr = Expression::unspanned(ExprKind::FnCall(fn_call));
        let mut context = TypeEnvironment::new();

        context.insert_function(crate::FnSignature {
            name: "my_fn".into(),
            arg_types: vec![],
            return_type: Rc::new(TypeExpression::NumberType),
        });

        let result = expr.type_check(&mut context);
        assert!(matches!(
            result,
            Err(crate::TypeCheckError::ArgumentCountMismatch { .. })
        ));
    }

    #[test]
    fn type_check_let_binding() {
        let bindings = LetBindings::new(vec![(
            "x".into(),
            Expression::rc(ExprKind::Lit(Literal::Number(1i32.into()))),
        )]);
        let body = Expression::rc(ExprKind::Var("x".into()));
        let let_expr = LetExpression::new(bindings, body);
        let expr = Expression::unspanned(ExprKind::Let(let_expr));
        let mut context = TypeEnvironment::new();

        let result = expr.type_check(&mut context);
        assert!(matches!(result, Ok(InferredType::Known(_))));
    }

    #[test]
    fn type_check_if_else_returns_then_or_else_type() {
        let condition = Expression::rc(ExprKind::Lit(Literal::Bool(true)));
        let then_expr = Expression::rc(ExprKind::Lit(Literal::Number(1i32.into())));
        let else_expr = Expression::rc(ExprKind::Lit(Literal::Number(2i32.into())));
        let if_else = IfElse::new(condition, then_expr, else_expr);
        let expr = Expression::unspanned(ExprKind::IfElse(if_else));

        let result = expr.type_check(&mut TypeEnvironment::new());
        assert!(matches!(result, Ok(InferredType::Known(_))));
    }

    #[test]
    fn type_check_if_else_condition_must_be_bool() {
        let condition = Expression::rc(ExprKind::Lit(Literal::Number(1i32.into())));
        let then_expr = Expression::rc(ExprKind::Lit(Literal::Number(1i32.into())));
        let else_expr = Expression::rc(ExprKind::Lit(Literal::Number(2i32.into())));
        let if_else = IfElse::new(condition, then_expr, else_expr);
        let expr = Expression::unspanned(ExprKind::IfElse(if_else));

        let result = expr.type_check(&mut TypeEnvironment::new());
        assert!(matches!(
            result,
            Err(crate::TypeCheckError::TypeMismatch { .. })
        ));
    }

    #[test]
    fn type_check_number_arithmetic_op() {
        let left = Expression::rc(ExprKind::Lit(Literal::Number(1i32.into())));
        let right = Expression::rc(ExprKind::Lit(Literal::Number(2i32.into())));

        for op in ["+", "-", "*", "/", "%"] {
            let expr =
                Expression::unspanned(ExprKind::OpCall(op.into(), left.clone(), right.clone()));
            let result = expr.type_check(&mut TypeEnvironment::new());
            assert!(
                matches!(result, Ok(InferredType::Known(_))),
                "Failed for operator: {}",
                op
            );
        }
    }

    #[test]
    fn type_check_comparison_op_returns_bool() {
        let left = Expression::rc(ExprKind::Lit(Literal::Number(1i32.into())));
        let right = Expression::rc(ExprKind::Lit(Literal::Number(2i32.into())));

        for op in [">", "<", ">=", "<="] {
            let expr =
                Expression::unspanned(ExprKind::OpCall(op.into(), left.clone(), right.clone()));
            let result = expr.type_check(&mut TypeEnvironment::new());
            assert!(
                matches!(result, Ok(InferredType::Known(_))),
                "Failed for operator: {}",
                op
            );
        }
    }

    #[test]
    fn type_check_equality_op_returns_bool() {
        let left = Expression::rc(ExprKind::Lit(Literal::Number(1i32.into())));
        let right = Expression::rc(ExprKind::Lit(Literal::Number(2i32.into())));

        for op in ["==", "!="] {
            let expr =
                Expression::unspanned(ExprKind::OpCall(op.into(), left.clone(), right.clone()));
            let result = expr.type_check(&mut TypeEnvironment::new());
            assert!(
                matches!(result, Ok(InferredType::Known(_))),
                "Failed for operator: {}",
                op
            );
        }
    }

    #[test]
    fn type_check_logical_op_both_operands_must_be_bool() {
        let left = Expression::rc(ExprKind::Lit(Literal::Number(1i32.into())));
        let right = Expression::rc(ExprKind::Lit(Literal::Bool(true)));

        for op in ["&&", "||"] {
            let expr =
                Expression::unspanned(ExprKind::OpCall(op.into(), left.clone(), right.clone()));
            let result = expr.type_check(&mut TypeEnvironment::new());
            assert!(
                matches!(result, Err(crate::TypeCheckError::TypeMismatch { .. })),
                "Should fail for operator: {}",
                op
            );
        }
    }

    #[test]
    fn type_check_query() {
        let binding = QueryBinding::new(
            vec!["person".into()],
            Expression::rc(ExprKind::Symbol("Person".into())),
        );
        let bindings = QueryBindings::new(vec![binding]);
        let guards = QueryGuards::new(vec![]);
        let production = Expression::rc(ExprKind::Symbol("name".into()));
        let query = Query::new(bindings, guards, production);

        let expr = Expression::unspanned(ExprKind::Query(query));
        let result = expr.type_check(&mut TypeEnvironment::new());

        assert!(matches!(result, Ok(InferredType::Known(_))));
    }

    #[test]
    fn type_check_query_with_guard() {
        let binding = QueryBinding::new(
            vec!["person".into()],
            Expression::rc(ExprKind::Symbol("Person".into())),
        );
        let bindings = QueryBindings::new(vec![binding]);

        let guard = Expression::rc(ExprKind::Lit(Literal::Bool(true)));
        let guards = QueryGuards::new(vec![guard]);

        let production = Expression::rc(ExprKind::Symbol("name".into()));
        let query = Query::new(bindings, guards, production);

        let expr = Expression::unspanned(ExprKind::Query(query));
        let result = expr.type_check(&mut TypeEnvironment::new());

        assert!(matches!(result, Ok(InferredType::Known(_))));
    }

    #[test]
    fn type_check_query_guard_must_be_bool() {
        let binding = QueryBinding::new(
            vec!["person".into()],
            Expression::rc(ExprKind::Symbol("Person".into())),
        );
        let bindings = QueryBindings::new(vec![binding]);

        let guard = Expression::rc(ExprKind::Lit(Literal::Number(42i32.into())));
        let guards = QueryGuards::new(vec![guard]);

        let production = Expression::rc(ExprKind::Symbol("name".into()));
        let query = Query::new(bindings, guards, production);

        let expr = Expression::unspanned(ExprKind::Query(query));
        let result = expr.type_check(&mut TypeEnvironment::new());

        assert!(matches!(
            result,
            Err(crate::TypeCheckError::TypeMismatch { .. })
        ));
    }

    #[test]
    fn type_check_symbol() {
        let expr = Expression::unspanned(ExprKind::Symbol("my_symbol".into()));
        let result = expr.type_check(&mut TypeEnvironment::new());

        assert!(matches!(result, Ok(InferredType::Known(_))));
    }

    #[test]
    fn type_check_const_or_type_ref() {
        let mut context = TypeEnvironment::new();
        context.insert_type_def("Person".into(), Rc::new(TypeExpression::SymbolType));

        let expr = Expression::unspanned(ExprKind::ConstOrTypeRef("Person".into()));
        let result = expr.type_check(&mut context);

        assert!(matches!(result, Ok(InferredType::Known(_))));
    }

    #[test]
    fn type_check_undefined_const_or_type_ref() {
        let expr = Expression::unspanned(ExprKind::ConstOrTypeRef("UndefinedType".into()));
        let result = expr.type_check(&mut TypeEnvironment::new());

        assert!(matches!(
            result,
            Err(crate::TypeCheckError::UndefinedVariable(_, _))
        ));
    }

    #[test]
    fn type_check_map_literal() {
        let kv_pairs = rogato_common::ast::literal::TupleItems::from(vec![
            Rc::new(rogato_common::ast::literal::MapKVPair {
                key: Expression::rc(ExprKind::Lit(Literal::String("key1".to_string()))),
                value: Expression::rc(ExprKind::Lit(Literal::Number(1i32.into()))),
            }),
            Rc::new(rogato_common::ast::literal::MapKVPair {
                key: Expression::rc(ExprKind::Lit(Literal::String("key2".to_string()))),
                value: Expression::rc(ExprKind::Lit(Literal::Number(2i32.into()))),
            }),
        ]);
        let literal = Literal::Map(kv_pairs);
        let expr = Expression::unspanned(ExprKind::Lit(literal));

        let result = expr.type_check(&mut TypeEnvironment::new());
        assert!(matches!(result, Ok(InferredType::Known(_))));
    }

    #[test]
    fn type_check_lambda() {
        use rogato_common::ast::lambda::{Lambda, LambdaArgs};

        let args = LambdaArgs::new(vec![]);
        let body = Expression::rc(ExprKind::Lit(Literal::Number(1i32.into())));
        let variant = rogato_common::ast::lambda::LambdaVariant::new(args, body);
        let lambda = Rc::new(Lambda::new(vec![Rc::new(variant)]));
        let expr = Expression::unspanned(ExprKind::Lambda(lambda));

        let result = expr.type_check(&mut TypeEnvironment::new());
        assert!(matches!(result, Ok(InferredType::Known(_))));
    }

    #[test]
    fn type_check_lambda_with_args() {
        // Lambda: (x -> x == 1)
        // Should infer as FunctionType(Unknown -> Bool)
        // We use == instead of + because + calls ensure_type which rejects
        // Unknown (the inferred type for a bare Var pattern) as not NumberType,
        // whereas == does not enforce operand types.
        let args = LambdaArgs::new(vec![Rc::new(Pattern::Var("x".into()))]);
        let body = Expression::rc(ExprKind::OpCall(
            "==".into(),
            Expression::rc(ExprKind::Var("x".into())),
            Expression::rc(ExprKind::Lit(Literal::Number(1i32.into()))),
        ));
        let variant = LambdaVariant::new(args, body);
        let lambda = Rc::new(Lambda::new(vec![Rc::new(variant)]));
        let expr = Expression::unspanned(ExprKind::Lambda(lambda));

        let mut env = TypeEnvironment::new();
        let result = expr.type_check(&mut env);
        assert!(result.is_ok());
        let inferred = result.unwrap();
        assert!(matches!(inferred, InferredType::Known(_)));
    }

    #[test]
    fn type_check_lambda_body_sees_args() {
        // Lambda: (x -> x)  — identity function
        // The body references 'x' which is a lambda arg — should NOT error as undefined
        let args = LambdaArgs::new(vec![Rc::new(Pattern::Var("x".into()))]);
        let body = Expression::rc(ExprKind::Var("x".into()));
        let variant = LambdaVariant::new(args, body);
        let lambda = Rc::new(Lambda::new(vec![Rc::new(variant)]));
        let expr = Expression::unspanned(ExprKind::Lambda(lambda));

        let mut env = TypeEnvironment::new();
        let result = expr.type_check(&mut env);
        // This should succeed — 'x' should be in scope from the lambda arg
        assert!(
            result.is_ok(),
            "Lambda body should see lambda args: {:?}",
            result
        );
    }

    #[test]
    fn type_infer_lambda_with_number_pattern() {
        // Lambda with number pattern: (0 -> true)
        let args = LambdaArgs::new(vec![Rc::new(Pattern::Number(0i32.into()))]);
        let body = Expression::rc(ExprKind::Lit(Literal::Bool(true)));
        let variant = LambdaVariant::new(args, body);
        let lambda = Rc::new(Lambda::new(vec![Rc::new(variant)]));
        let expr = Expression::unspanned(ExprKind::Lambda(lambda));

        let inferrer = TypeInferrer::new();
        let result = inferrer.infer_expression(&expr);
        assert!(matches!(result, InferredType::Known(_)));

        if let InferredType::Known(type_expr) = result {
            match type_expr.deref() {
                TypeExpression::FunctionType(arg_types, return_type) => {
                    // Arg type should be NumberType (inferred from pattern)
                    assert_eq!(arg_types.len(), 1);
                    // Return type should be BoolType
                    assert!(matches!(return_type.deref(), TypeExpression::BoolType));
                }
                _ => panic!("Expected FunctionType"),
            }
        }
    }

    #[test]
    fn type_check_inline_fn_def() {
        use rogato_common::ast::fn_def::FnDefArgs;
        use rogato_common::ast::fn_def::{FnDef, FnDefBody};

        let args = FnDefArgs::new(vec![]);
        let body = Rc::new(FnDefBody::RogatoFn(Expression::rc(ExprKind::Lit(
            Literal::Number(1i32.into()),
        ))));
        let fn_def = FnDef::new("my_fn", args, body);
        let expr = Expression::unspanned(ExprKind::InlineFnDef(fn_def));

        let result = expr.type_check(&mut TypeEnvironment::new());
        assert!(result.is_ok());
    }
}
