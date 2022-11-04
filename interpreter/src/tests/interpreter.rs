use crate::{EvalContext, Evaluate};
use rogato_common::val;
use rogato_parser::{parse_expr, ParserContext};
use rust_decimal_macros::dec;

#[test]
fn basic_arithmetic() {
    let expressions_and_values = [
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

    let mut eval_ctx = EvalContext::new();
    let parser_ctx = ParserContext::new();

    for (code, value) in expressions_and_values.iter() {
        let ast = parse_expr(code, &parser_ctx).unwrap();
        assert_eq!(ast.evaluate(&mut eval_ctx), Ok(value.clone()))
    }
}

#[test]
fn multiplication() {
    let mut eval_ctx = EvalContext::new();
    let parser_ctx = ParserContext::new();

    for i in -100..100 {
        let a = i * 10;
        let b = i * 100;
        let ast = parse_expr(format!("{} * {}", a, b).as_str(), &parser_ctx).unwrap();
        assert_eq!(ast.evaluate(&mut eval_ctx), Ok(val::number(a * b)));
    }
}

#[test]
fn string_literals() {
    let mut eval_ctx = EvalContext::new();
    let parser_ctx = ParserContext::new();

    let string_literals = [
        "",
        "!",
        " ",
        "  ",
        "Hello, World!",
        "My name is John Connor",
    ];

    for string_lit in string_literals.iter() {
        let parse_result = parse_expr(format!("{:?}", string_lit).as_str(), &parser_ctx);
        assert!(parse_result.is_ok());
        if let Ok(ast) = parse_result {
            assert_eq!(ast.evaluate(&mut eval_ctx), Ok(val::string(string_lit)));
        }
    }
}

#[test]
fn let_expressions() {
    let mut eval_ctx = EvalContext::new();
    let parser_ctx = ParserContext::new();

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
        &parser_ctx,
    )
    .unwrap();

    assert_eq!(
        ast.evaluate(&mut eval_ctx),
        Ok(val::tuple([
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
        &parser_ctx,
    )
    .unwrap();

    assert_eq!(
        ast.evaluate(&mut eval_ctx),
        Ok(val::tuple([val::number(3), val::number(6)]))
    )
}

#[test]
fn equality() {
    let not_equal = [
        ("1", "2"),
        ("1", "\"foo\""),
        ("^foo", "^bar"),
        ("{1,2}", "{2,1}"),
        ("{1,2,3}", "{3,4,5}"),
        ("{1, 1 + 1, 2 + 1}", "{3,4,5}"),
        ("[^foo, ^bar, 1 + 2]", "[^bar, 3, ^foo]"),
    ];

    let equal = [
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

    let mut eval_ctx = EvalContext::new();
    let parser_ctx = ParserContext::new();

    for (left, right) in not_equal.iter() {
        let code = format!("{} == {}", left, right);
        let ast = parse_expr(code.as_str(), &parser_ctx).unwrap();
        assert_eq!(ast.evaluate(&mut eval_ctx), Ok(val::bool(false)));

        let code = format!("{} != {}", left, right);
        let ast = parse_expr(code.as_str(), &parser_ctx).unwrap();
        assert_eq!(ast.evaluate(&mut eval_ctx), Ok(val::bool(true)));
    }

    for (left, right) in equal.iter() {
        let code = format!("{} == {}", left, right);
        let ast = parse_expr(code.as_str(), &parser_ctx).unwrap();
        assert_eq!(ast.evaluate(&mut eval_ctx), Ok(val::bool(true)));

        let code = format!("{} != {}", left, right);
        let ast = parse_expr(code.as_str(), &parser_ctx).unwrap();
        assert_eq!(ast.evaluate(&mut eval_ctx), Ok(val::bool(false)));
    }
}

#[test]
fn std_module() {
    let code_with_vals = [
        (
            "Std.range 10",
            val::list([
                val::number(0),
                val::number(1),
                val::number(2),
                val::number(3),
                val::number(4),
                val::number(5),
                val::number(6),
                val::number(7),
                val::number(8),
                val::number(9),
            ]),
        ),
        (
            "Std.range 1 10",
            val::list([
                val::number(1),
                val::number(2),
                val::number(3),
                val::number(4),
                val::number(5),
                val::number(6),
                val::number(7),
                val::number(8),
                val::number(9),
            ]),
        ),
        ("Std.random 0", val::number(0)),
        ("random 0", val::number(0)),
        ("random 0 0", val::number(0)),
        ("random 1 1", val::number(1)),
        ("random 1.0 1", val::number(1)),
        ("random 1 1.0", val::number(1)),
        ("random 1.0 1.0", val::number(1)),
    ];

    let mut eval_ctx = EvalContext::new();
    let parser_ctx = ParserContext::new();

    for (code, val) in code_with_vals.iter() {
        let ast = parse_expr(code, &parser_ctx).unwrap();
        assert_eq!(ast.evaluate(&mut eval_ctx), Ok(val.clone()));
    }
}

#[test]
fn std_math_module() {
    let code_with_vals = [
        ("Std.Math.abs -10", val::number(10)),
        ("Std.Math.abs (10 * -10)", val::number(100)),
        ("Std.Math.abs (100 - 1000)", val::number(900)),
        ("abs -10", val::number(10)),
        ("abs (10 * -10)", val::number(100)),
        ("abs (100 - 1000)", val::number(900)),
        ("Std.Math.sqrt 9", val::some(val::number(3))),
        (
            "sqrt 2",
            val::some(val::number(dec!(1.4142135623730950488016887242))),
        ),
        ("sqrt 4", val::some(val::number(2))),
        ("sqrt 100", val::some(val::number(10))),
        ("sqrt 10000", val::some(val::number(100))),
    ];

    let mut eval_ctx = EvalContext::new();
    let parser_ctx = ParserContext::new();

    for (code, val) in code_with_vals.iter() {
        let ast = parse_expr(code, &parser_ctx).unwrap();
        assert_eq!(ast.evaluate(&mut eval_ctx), Ok(val.clone()));
    }
}

#[test]
fn std_string_module() {
    let code_with_vals = [
        ("Std.String.length \"\"", val::number(0)),
        ("Std.String.length \" \"", val::number(1)),
        ("Std.String.length \"hello\"", val::number(5)),
        ("Std.String.length \"hello, world\"", val::number(12)),
    ];

    let mut eval_ctx = EvalContext::new();
    let parser_ctx = ParserContext::new();

    for (code, val) in code_with_vals.iter() {
        let ast = parse_expr(code, &parser_ctx).unwrap();
        assert_eq!(ast.evaluate(&mut eval_ctx), Ok(val.clone()));
    }
}

#[test]
fn std_list_module() {
    let code_with_vals = [
        (
            "Std.List.map [1,2,3] ^inspect",
            val::list([val::string("1"), val::string("2"), val::string("3")]),
        ),
        (
            "Std.List.map [1,2,3] (x -> inspect x)",
            val::list([val::string("1"), val::string("2"), val::string("3")]),
        ),
        (
            "let
                insp   = x -> inspect x
                add10  = x -> x + 10
                add1 x = x + 1
                add2 x = (add1 (add1 x))
             in
                {
                    Std.List.map [1,2,3] insp,
                    Std.List.map [10,20,30] ^inspect,
                    Std.List.map [10,20,30] add10,
                    add10 1,
                    add1 1000,
                    add2 0,
                    add2 1
                }",
            val::tuple([
                val::list([val::string("1"), val::string("2"), val::string("3")]),
                val::list([val::string("10"), val::string("20"), val::string("30")]),
                val::list([val::number(20), val::number(30), val::number(40)]),
                val::number(11),
                val::number(1001),
                val::number(2),
                val::number(3),
            ]),
        ),
        (
            "[1 :: [2,3,4]]",
            val::list([
                val::number(1),
                val::number(2),
                val::number(3),
                val::number(4),
            ]),
        ),
        (
            "[{1,2,3} :: [2,3,4]]",
            val::list([
                val::tuple([val::number(1), val::number(2), val::number(3)]),
                val::number(2),
                val::number(3),
                val::number(4),
            ]),
        ),
    ];

    let mut eval_ctx = EvalContext::new();
    let parser_ctx = ParserContext::new();

    for (code, val) in code_with_vals.iter() {
        let ast = parse_expr(code, &parser_ctx).unwrap();
        assert_eq!(ast.evaluate(&mut eval_ctx), Ok(val.clone()));
    }
}

#[test]
fn if_else_expr() {
    let code_with_vals = [
        (
            "if (2 == 2) then ^equal else ^unequal",
            val::symbol("equal"),
        ),
        (
            "if (2 == 3) then ^equal else ^unequal",
            val::symbol("unequal"),
        ),
        ("if (2 < 3) then ^ok else ^fail", val::symbol("ok")),
        ("if (2 > 3) then ^ok else ^fail", val::symbol("fail")),
        (
            "let
                greater a b = if (a > b) then a else b
             in
                greater 10 20",
            val::number(20),
        ),
        (
            "let
                greater a b = if (a > b) then a else b
             in
                greater 100 20",
            val::number(100),
        ),
    ];

    let mut eval_ctx = EvalContext::new();
    let parser_ctx = ParserContext::new();

    for (code, val) in code_with_vals.iter() {
        let ast = parse_expr(code, &parser_ctx).unwrap();
        assert_eq!(ast.evaluate(&mut eval_ctx), Ok(val.clone()));
    }
}

#[test]
fn lambda_closures() {
    let code_with_vals = vec![(
        "let
            f x = [1, 2, 3] |> Std.List.map (y -> x + y)
        in
            {f 1, f 10, f 100}",
        val::tuple(vec![
            val::list(vec![val::number(2), val::number(3), val::number(4)]),
            val::list(vec![val::number(11), val::number(12), val::number(13)]),
            val::list(vec![val::number(101), val::number(102), val::number(103)]),
        ]),
    )];

    let mut eval_ctx = EvalContext::new();
    let parser_ctx = ParserContext::new();

    for (code, val) in code_with_vals.iter() {
        let ast = parse_expr(code, &parser_ctx).unwrap();
        assert_eq!(ast.evaluate(&mut eval_ctx), Ok(val.clone()));
    }
}
