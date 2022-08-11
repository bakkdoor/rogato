use crate::{EvalContext, Evaluate};
use rogato_common::val;
use rogato_parser::parse_expr;

#[test]
fn basic_arithmetic() {
    let mut ctx = EvalContext::new();
    let expressions_and_values = vec![
        ("3 + 4", val::int64(7)),
        ("-3 + 4", val::int64(1)),
        ("100 - 90", val::int64(10)),
        ("-100 - 90", val::int64(-190)),
        ("10 * 500", val::int64(5000)),
        ("-10 * 500", val::int64(-5000)),
        ("10 * -500", val::int64(-5000)),
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
fn multiplication() {
    let mut ctx = EvalContext::new();

    for i in -100..100 {
        let a = i * 10;
        let b = i * 100;
        let ast = parse_expr(format!("{} * {}", a, b).as_str()).unwrap();
        assert_eq!(ast.evaluate(&mut ctx), Ok(val::int64(a * b)));
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

#[test]
fn let_expressions() {
    let mut ctx = EvalContext::new();
    let ast = parse_expr(
        "let
            f x =
                x + 1
            g x y =
                x - (f (x / y))
            x =
                f 101
         in
            {x, f x, g x 10}",
    )
    .unwrap();

    assert_eq!(
        ast.evaluate(&mut ctx),
        Ok(val::tuple(vec![
            val::int64(102),
            val::int64(103),
            val::int64(91)
        ]))
    );

    let ast = parse_expr(
        "let
            add a b = a + b
            mul a b = a * b
         in
            { add 1 2, mul 2 3 }",
    )
    .unwrap();

    assert_eq!(
        ast.evaluate(&mut ctx),
        Ok(val::tuple(vec![val::int64(3), val::int64(6)]))
    )
}

#[test]
fn std_math_module() {
    let mut ctx = EvalContext::new();
    let code_with_vals = vec![
        ("Std.Math.abs -10", val::int64(10)),
        ("Std.Math.abs (10 * -10)", val::int64(100)),
        ("Std.Math.abs (100 - 1000)", val::int64(900)),
    ];

    for (code, val) in code_with_vals.iter() {
        let ast = parse_expr(code).unwrap();
        assert_eq!(ast.evaluate(&mut ctx), Ok(val.clone()));
    }
}
