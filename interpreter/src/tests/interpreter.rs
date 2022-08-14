use crate::{EvalContext, Evaluate};
use rogato_common::val;
use rogato_parser::{parse_expr, ParserContext};

#[test]
fn basic_arithmetic() {
    let expressions_and_values = vec![
        ("3 + 4", val::decimal(7)),
        ("-3 + 4", val::decimal(1)),
        ("100 - 90", val::decimal(10)),
        ("-100 - 90", val::decimal(-190)),
        ("10 * 500", val::decimal(5000)),
        ("-10 * 500", val::decimal(-5000)),
        ("10 * -500", val::decimal(-5000)),
        ("10 / 3", val::decimal_str("3.3333333333333333333333333333")),
        ("500 / 25", val::decimal(20)),
        ("10 % 3", val::decimal(1)),
        ("500 % 28", val::decimal(24)),
        ("10 ^ 2", val::decimal(100)),
        ("10 ^ 10", val::decimal(10000000000i64)),
    ];

    let mut ctx = EvalContext::new();
    let p_ctx = ParserContext::new();

    for (code, value) in expressions_and_values.iter() {
        let ast = parse_expr(code, &p_ctx).unwrap();
        assert_eq!(ast.evaluate(&mut ctx), Ok(value.clone()))
    }
}

#[test]
fn multiplication() {
    let mut ctx = EvalContext::new();
    let p_ctx = ParserContext::new();

    for i in -100..100 {
        let a = i * 10;
        let b = i * 100;
        let ast = parse_expr(format!("{} * {}", a, b).as_str(), &p_ctx).unwrap();
        assert_eq!(ast.evaluate(&mut ctx), Ok(val::decimal(a * b)));
    }
}

#[test]
fn string_literals() {
    let mut ctx = EvalContext::new();
    let p_ctx = ParserContext::new();

    let string_literals = vec![
        "",
        "!",
        " ",
        "  ",
        "Hello, World!",
        "My name is John Connor",
    ];

    for string_lit in string_literals.iter() {
        let parse_result = parse_expr(format!("{:?}", string_lit).as_str(), &p_ctx);
        assert!(parse_result.is_ok());
        if let Ok(ast) = parse_result {
            assert_eq!(ast.evaluate(&mut ctx), Ok(val::string(string_lit)));
        }
    }
}

#[test]
fn let_expressions() {
    let mut ctx = EvalContext::new();
    let p_ctx = ParserContext::new();

    let ast = parse_expr(
        "let
            f x =
                x + 1
            g y z =
                y - (f (y / z))
            x =
                f 101
         in
            {x, f x, g x 10}",
        &p_ctx,
    )
    .unwrap();

    assert_eq!(
        ast.evaluate(&mut ctx),
        Ok(val::tuple(vec![
            val::decimal(102),
            val::decimal(103),
            val::decimal_str("90.8")
        ]))
    );

    let ast = parse_expr(
        "let
            add a b = a + b
            mul a b = a * b
         in
            { add 1 2, mul 2 3 }",
        &p_ctx,
    )
    .unwrap();

    assert_eq!(
        ast.evaluate(&mut ctx),
        Ok(val::tuple(vec![val::decimal(3), val::decimal(6)]))
    )
}

#[test]
fn std_math_module() {
    let code_with_vals = vec![
        ("Std.Math.abs -10", val::decimal(10)),
        ("Std.Math.abs (10 * -10)", val::decimal(100)),
        ("Std.Math.abs (100 - 1000)", val::decimal(900)),
        ("abs -10", val::decimal(10)),
        ("abs (10 * -10)", val::decimal(100)),
        ("abs (100 - 1000)", val::decimal(900)),
    ];

    let mut ctx = EvalContext::new();
    let p_ctx = ParserContext::new();

    for (code, val) in code_with_vals.iter() {
        let ast = parse_expr(code, &p_ctx).unwrap();
        assert_eq!(ast.evaluate(&mut ctx), Ok(val.clone()));
    }
}

#[test]
fn std_string_module() {
    let code_with_vals = vec![
        ("Std.String.length \"\"", val::decimal(0)),
        ("Std.String.length \" \"", val::decimal(1)),
        ("Std.String.length \"hello\"", val::decimal(5)),
        ("Std.String.length \"hello, world\"", val::decimal(12)),
    ];

    let mut ctx = EvalContext::new();
    let p_ctx = ParserContext::new();

    for (code, val) in code_with_vals.iter() {
        let ast = parse_expr(code, &p_ctx).unwrap();
        assert_eq!(ast.evaluate(&mut ctx), Ok(val.clone()));
    }
}
