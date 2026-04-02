use rogato_common::ast::{
    expression::Expression, literal::Literal, type_expression::TypeExpression, Identifier,
    VarIdentifier,
};
use std::rc::Rc;

use crate::{InferredType, TypeInferrer};

#[test]
fn infer_number_literal() {
    let inferrer = TypeInferrer::new();
    let literal = Literal::Number(42i32.into());
    let expr = Expression::Lit(literal);

    let result = inferrer.infer_expression(&expr);
    assert!(matches!(result, InferredType::Known(_)));
}

#[test]
fn infer_string_literal() {
    let inferrer = TypeInferrer::new();
    let literal = Literal::String("hello".to_string());
    let expr = Expression::Lit(literal);

    let result = inferrer.infer_expression(&expr);
    assert!(matches!(result, InferredType::Known(_)));
}

#[test]
fn infer_bool_literal() {
    let inferrer = TypeInferrer::new();
    let literal = Literal::Bool(true);
    let expr = Expression::Lit(literal);

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
    let left_expr = Rc::new(Expression::Lit(left));
    let right_expr = Rc::new(Expression::Lit(right));
    let expr = Expression::OpCall(op, left_expr, right_expr);

    let result = inferrer.infer_expression(&expr);
    assert!(matches!(result, InferredType::Known(_)));
}

#[test]
fn infer_undefined_variable() {
    let inferrer = TypeInferrer::new();

    let var: VarIdentifier = "undefined_var".into();
    let expr = Expression::Var(var);

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
    let expr = Expression::Var(var);

    let result = inferrer.infer_expression(&expr);
    assert!(matches!(result, InferredType::Known(_)));
}

#[test]
fn infer_list_literal() {
    let inferrer = TypeInferrer::new();

    let items: Vec<Rc<Expression>> = vec![
        Rc::new(Expression::Lit(Literal::Number(1i32.into()))),
        Rc::new(Expression::Lit(Literal::Number(2i32.into()))),
    ];

    let literal = Literal::List(rogato_common::ast::literal::TupleItems::from(items));
    let expr = Expression::Lit(literal);

    let result = inferrer.infer_expression(&expr);
    assert!(matches!(result, InferredType::Known(_)));
}

#[test]
fn infer_tuple_literal() {
    let inferrer = TypeInferrer::new();

    let items: Vec<Rc<Expression>> = vec![
        Rc::new(Expression::Lit(Literal::Number(1i32.into()))),
        Rc::new(Expression::Lit(Literal::String("hello".to_string()))),
    ];

    let literal = Literal::Tuple(rogato_common::ast::literal::TupleItems::from(items));
    let expr = Expression::Lit(literal);

    let result = inferrer.infer_expression(&expr);
    assert!(matches!(result, InferredType::Known(_)));
}

#[test]
fn infer_comparison_op() {
    let inferrer = TypeInferrer::new();

    let left = Literal::Number(1i32.into());
    let right = Literal::Number(2i32.into());

    let op: Identifier = ">".into();
    let left_expr = Rc::new(Expression::Lit(left));
    let right_expr = Rc::new(Expression::Lit(right));
    let expr = Expression::OpCall(op, left_expr, right_expr);

    let result = inferrer.infer_expression(&expr);
    assert!(matches!(result, InferredType::Known(_)));
}

#[test]
fn infer_logical_op() {
    let inferrer = TypeInferrer::new();

    let left = Literal::Bool(true);
    let right = Literal::Bool(false);

    let op: Identifier = "&&".into();
    let left_expr = Rc::new(Expression::Lit(left));
    let right_expr = Rc::new(Expression::Lit(right));
    let expr = Expression::OpCall(op, left_expr, right_expr);

    let result = inferrer.infer_expression(&expr);
    assert!(matches!(result, InferredType::Known(_)));
}
