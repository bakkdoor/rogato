use crate::rogato::ast::helpers::parse_expr;
use crate::rogato::db::val;
use crate::rogato::interpreter::{EvalContext, Evaluate};

#[test]
fn basic_arithmetic() {
    let mut ctx = EvalContext::new();
    let expressions_and_values = vec![
        ("3 + 4", val::int64(7)),
        ("100 - 90", val::int64(10)),
        ("10 * 500", val::int64(5000)),
        ("10 / 3", val::int64(3)),
        ("500 / 25", val::int64(20)),
        ("10 % 3", val::int64(1)),
        ("500 % 28", val::int64(24)),
        ("10 ^ 2", val::int64(100)),
        ("10 ^ 10", val::int64(10_000_000_000)),
    ];

    for (code, value) in expressions_and_values.iter() {
        let ast = parse_expr(code).unwrap();
        assert_eq!(ast.evaluate(&mut ctx), Ok(value.clone()))
    }
}

#[test]
fn string_literals() {
    let mut ctx = EvalContext::new();

    let string_literals = vec![
        "",
        "!",
        " ",
        "  ",
        "Hello, World!",
        "My name is John Connor",
    ];

    for string_lit in string_literals.iter() {
        let parse_result = parse_expr(format!("{:?}", string_lit).as_str());
        assert!(parse_result.is_ok());
        if let Ok(ast) = parse_result {
            assert_eq!(ast.evaluate(&mut ctx), Ok(val::string(string_lit)));
        }
    }
}
