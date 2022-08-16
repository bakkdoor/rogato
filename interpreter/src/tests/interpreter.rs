use crate::{EvalContext, Evaluate};
use rogato_common::val;
use rogato_parser::{parse_expr, ParserContext};

#[test]
fn basic_arithmetic() {
    let expressions_and_values = vec![
        ("3 + 4", val::number(7)),
        ("-3 + 4", val::number(1)),
        ("100 - 90", val::number(10)),
        ("-100 - 90", val::number(-190)),
        ("10 * 500", val::number(5000)),
        ("-10 * 500", val::number(-5000)),
        ("10 * -500", val::number(-5000)),
        ("10 / 3", val::decimal_str("3.3333333333333333333333333333")),
        ("500 / 25", val::number(20)),
        ("10 % 3", val::number(1)),
        ("500 % 28", val::number(24)),
        ("10 ^ 2", val::number(100)),
        ("10 ^ 10", val::number(10000000000i64)),
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
        assert_eq!(ast.evaluate(&mut ctx), Ok(val::number(a * b)));
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
            val::number(102),
            val::number(103),
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
        Ok(val::tuple(vec![val::number(3), val::number(6)]))
    )
}

#[test]
fn std_math_module() {
    let code_with_vals = vec![
        ("Std.Math.abs -10", val::number(10)),
        ("Std.Math.abs (10 * -10)", val::number(100)),
        ("Std.Math.abs (100 - 1000)", val::number(900)),
        ("abs -10", val::number(10)),
        ("abs (10 * -10)", val::number(100)),
        ("abs (100 - 1000)", val::number(900)),
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
        ("Std.String.length \"\"", val::number(0)),
        ("Std.String.length \" \"", val::number(1)),
        ("Std.String.length \"hello\"", val::number(5)),
        ("Std.String.length \"hello, world\"", val::number(12)),
    ];

    let mut ctx = EvalContext::new();
    let p_ctx = ParserContext::new();

    for (code, val) in code_with_vals.iter() {
        let ast = parse_expr(code, &p_ctx).unwrap();
        assert_eq!(ast.evaluate(&mut ctx), Ok(val.clone()));
    }
}

#[test]
fn equality() {
    let not_equal = vec![
        ("1", "2"),
        ("1", "\"foo\""),
        ("^foo", "^bar"),
        ("{1,2}", "{2,1}"),
        ("{1,2,3}", "{3,4,5}"),
        ("{1, 1 + 1, 2 + 1}", "{3,4,5}"),
        ("[^foo, ^bar, 1 + 2]", "[^bar, 3, ^foo]"),
    ];

    let equal = vec![
        ("1", "1"),
        ("\"foo\"", "\"foo\""),
        ("^foo", "^foo"),
        ("{1,2,3}", "{1,2,3}"),
        ("{1, 1 + 1, 2 + 1}", "{1,2,3}"),
        ("[]", "[]"),
        ("[1]", "[1]"),
        (
            "[^foo, ^bar, 1 + 2, \"hello\" ++ \", world!\"]",
            "[^foo, ^bar, 3, \"hello, world!\"]",
        ),
    ];

    let mut ctx = EvalContext::new();
    let p_ctx = ParserContext::new();

    for (left, right) in not_equal.iter() {
        let code = format!("{} == {}", left, right);
        let ast = parse_expr(code.as_str(), &p_ctx).unwrap();
        assert_eq!(ast.evaluate(&mut ctx), Ok(val::bool(false)));

        let code = format!("{} != {}", left, right);
        let ast = parse_expr(code.as_str(), &p_ctx).unwrap();
        assert_eq!(ast.evaluate(&mut ctx), Ok(val::bool(true)));
    }

    for (left, right) in equal.iter() {
        let code = format!("{} == {}", left, right);
        let ast = parse_expr(code.as_str(), &p_ctx).unwrap();
        assert_eq!(ast.evaluate(&mut ctx), Ok(val::bool(true)));

        let code = format!("{} != {}", left, right);
        let ast = parse_expr(code.as_str(), &p_ctx).unwrap();
        assert_eq!(ast.evaluate(&mut ctx), Ok(val::bool(false)));
    }
}

#[test]
fn std_list_module() {
    let code_with_vals = vec![
        (
            "Std.List.map [1,2,3] ^inspect",
            val::list(vec![val::string("1"), val::string("2"), val::string("3")]),
        ),
        (
            "Std.List.map [1,2,3] (x -> inspect x)",
            val::list(vec![val::string("1"), val::string("2"), val::string("3")]),
        ),
        (
            "let
                insp  = x -> inspect x
                add10 = x -> x + 10
             in
                {
                    Std.List.map [1,2,3] insp,
                    Std.List.map [10,20,30] add10
                }",
            val::tuple(vec![
                val::list(vec![val::string("1"), val::string("2"), val::string("3")]),
                val::list(vec![val::number(20), val::number(30), val::number(40)]),
            ]),
        ),
    ];

    let mut ctx = EvalContext::new();
    let p_ctx = ParserContext::new();

    for (code, val) in code_with_vals.iter() {
        let ast = parse_expr(code, &p_ctx).unwrap();
        assert_eq!(ast.evaluate(&mut ctx), Ok(val.clone()));
    }
}
