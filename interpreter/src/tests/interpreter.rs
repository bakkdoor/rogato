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
            twice x = {x, x}
         in
            { add 1 2, mul 2 3, twice 1, twice 42.69, twice (twice ^wat) }",
        &parser_ctx,
    )
    .unwrap();

    assert_eq!(
        ast.evaluate(&mut eval_ctx),
        Ok(val::tuple([
            val::number(3),
            val::number(6),
            val::tuple([val::number(1), val::number(1)]),
            val::tuple([val::decimal_str("42.69"), val::decimal_str("42.69")]),
            val::tuple([
                val::tuple([val::symbol("wat"), val::symbol("wat")]),
                val::tuple([val::symbol("wat"), val::symbol("wat")])
            ])
        ]))
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
        ("true", "false"),
        ("false", "true"),
        ("2 == 2", "false"),
        ("2 != 3", "false"),
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
        ("false", "false"),
        ("true", "true"),
        ("2 == 2", "true"),
        ("2 != 3", "true"),
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
        ("toString 1", val::string("1")),
        ("toString 1.0", val::string("1.0")),
        ("toString 1.1", val::string("1.1")),
        ("toString 1.5987", val::string("1.5987")),
        ("toString {1.0, \"foo\"}", val::string("{ 1.0, \"foo\" }")),
        ("toString \"foo bar baz\"", val::string("foo bar baz")),
        (
            "toString (\"foo\" ++ \" \" ++ \"bar\")",
            val::string("foo bar"),
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
                cons = [{^foo, ^bar} :: (range 3)]
                cons2 list = [{^foo, ^bar} :: list]
             in
                {
                    Std.List.map [1,2,3] insp,
                    Std.List.map [10,20,30] ^inspect,
                    Std.List.map [10,20,30] add10,
                    add10 1,
                    add1 1000,
                    add2 0,
                    add2 1,
                    cons,
                    cons2 [],
                    cons2 [^a,^b,^c],
                    cons2 [\"hello, world\"],
                }",
            val::tuple([
                val::list([val::string("1"), val::string("2"), val::string("3")]),
                val::list([val::string("10"), val::string("20"), val::string("30")]),
                val::list([val::number(20), val::number(30), val::number(40)]),
                val::number(11),
                val::number(1001),
                val::number(2),
                val::number(3),
                val::list([
                    val::tuple([val::symbol("foo"), val::symbol("bar")]),
                    val::number(0),
                    val::number(1),
                    val::number(2),
                ]),
                val::list([val::tuple([val::symbol("foo"), val::symbol("bar")])]),
                val::list([
                    val::tuple([val::symbol("foo"), val::symbol("bar")]),
                    val::symbol("a"),
                    val::symbol("b"),
                    val::symbol("c"),
                ]),
                val::list([
                    val::tuple([val::symbol("foo"), val::symbol("bar")]),
                    val::string("hello, world"),
                ]),
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
fn std_map_module() {
    let code_with_vals = [
        ("{}", val::map([])),
        ("{1 : 2}", val::map([(val::number(1), val::number(2))])),
        (
            "{^foo : 2}",
            val::map([(val::symbol("foo"), val::number(2))]),
        ),
        (
            "{ ^foo : {^bar: { 2: ^hello_world }} }",
            val::map([(
                val::symbol("foo"),
                val::map([(
                    val::symbol("bar"),
                    val::map([(val::number(2), val::symbol("hello_world"))]),
                )]),
            )]),
        ),
        (
            "{{ ^hello: {^world, 1, 2, 3}} :: 1 : 2, ^foo : \"bar\"}",
            val::map([
                (
                    val::symbol("hello"),
                    val::tuple([
                        val::symbol("world"),
                        val::number(1),
                        val::number(2),
                        val::number(3),
                    ]),
                ),
                (val::number(1), val::number(2)),
                (val::symbol("foo"), val::string("bar")),
            ]),
        ),
        (
            "let
                map m f =
                    map_ {} m f
                map_ acc {} _ =
                    acc
                map_ acc {rest :: k : v} f =
                    map_ (acc |> Std.Map.insert (f k v)) rest f

                data = {
                    ^foo: ^bar,
                    ^bar: {1,2,3},
                    ^baz: {^hello, \"world\"},
                    1: {2, 3}
                }
        in
            map data (k v -> {{k,k}, v})",
            val::map([
                (
                    val::tuple([val::symbol("foo"), val::symbol("foo")]),
                    val::symbol("bar"),
                ),
                (
                    val::tuple([val::symbol("bar"), val::symbol("bar")]),
                    val::tuple([val::number(1), val::number(2), val::number(3)]),
                ),
                (
                    val::tuple([val::symbol("baz"), val::symbol("baz")]),
                    val::tuple([val::symbol("hello"), val::string("world")]),
                ),
                (
                    val::tuple([val::number(1), val::number(1)]),
                    val::tuple([val::number(2), val::number(3)]),
                ),
            ]),
        ),
        (
            "let
                map = {^foo: 1, ^bar: {1, 2, 3}, {1, 2}: ^wat}
            in
                {
                    map |> Std.Map.remove ^foo,
                    map |> Std.Map.remove ^bar,
                    map |> Std.Map.remove {1, 2},
                }",
            val::tuple([
                val::map([
                    (
                        val::symbol("bar"),
                        val::tuple([val::number(1), val::number(2), val::number(3)]),
                    ),
                    (
                        val::tuple([val::number(1), val::number(2)]),
                        val::symbol("wat"),
                    ),
                ]),
                val::map([
                    (val::symbol("foo"), val::number(1)),
                    (
                        val::tuple([val::number(1), val::number(2)]),
                        val::symbol("wat"),
                    ),
                ]),
                val::map([
                    (val::symbol("foo"), val::number(1)),
                    (
                        val::symbol("bar"),
                        val::tuple([val::number(1), val::number(2), val::number(3)]),
                    ),
                ]),
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
            g x = {f x, f (x * 2)}
            list = [10,100,1000] |> map ^g
        in
            {f 1, f 10, f 100, g 1, g 5, list}",
        val::tuple([
            val::list([val::number(2), val::number(3), val::number(4)]),
            val::list([val::number(11), val::number(12), val::number(13)]),
            val::list([val::number(101), val::number(102), val::number(103)]),
            val::tuple([
                val::list([val::number(2), val::number(3), val::number(4)]),
                val::list([val::number(3), val::number(4), val::number(5)]),
            ]),
            val::tuple([
                val::list([val::number(6), val::number(7), val::number(8)]),
                val::list([val::number(11), val::number(12), val::number(13)]),
            ]),
            val::list([
                val::tuple([
                    val::list([val::number(11), val::number(12), val::number(13)]),
                    val::list([val::number(21), val::number(22), val::number(23)]),
                ]),
                val::tuple([
                    val::list([val::number(101), val::number(102), val::number(103)]),
                    val::list([val::number(201), val::number(202), val::number(203)]),
                ]),
                val::tuple([
                    val::list([val::number(1001), val::number(1002), val::number(1003)]),
                    val::list([val::number(2001), val::number(2002), val::number(2003)]),
                ]),
            ]),
        ]),
    )];

    let mut eval_ctx = EvalContext::new();
    let parser_ctx = ParserContext::new();

    for (code, val) in code_with_vals.iter() {
        let ast = parse_expr(code, &parser_ctx).unwrap();
        assert_eq!(ast.evaluate(&mut eval_ctx), Ok(val.clone()));
    }
}

#[test]
fn patterns() {
    let code_with_vals = vec![
        (
            "let
                head [x :: _]  = x
                tail [_ :: xs] = xs
                list = [1 :: [2, \"foo\", false]]
            in
                { list, head list, tail list }",
            val::tuple([
                val::list([
                    val::number(1),
                    val::number(2),
                    val::string("foo"),
                    val::bool(false),
                ]),
                val::number(1),
                val::list([val::number(2), val::string("foo"), val::bool(false)]),
            ]),
        ),
        (
            "let
                fib 0 = 0
                fib 1 = 1
                fib x = (fib (x - 1)) + (fib (x - 2))
            in
                { fib 0, fib 1, fib 5, fib 10 }",
            val::tuple([
                val::number(0),
                val::number(1),
                val::number(5),
                val::number(55),
            ]),
        ),
        (
            "let
                reduce acc _ []        = acc
                reduce acc f [x]       = (f acc x)
                reduce acc f [x :: xs] = reduce (f acc x) f xs
            in
                {
                    reduce 0 (x y -> x + y) [1,2,3],
                    reduce 10 (x y -> x - y) [1,2,3],
                    reduce [] (acc x -> [{x,x} :: acc]) [1,^foo,2,^bar]
                }",
            val::tuple([
                val::number(6),
                val::number(4),
                val::list([
                    val::tuple([val::symbol("bar"), val::symbol("bar")]),
                    val::tuple([val::number(2), val::number(2)]),
                    val::tuple([val::symbol("foo"), val::symbol("foo")]),
                    val::tuple([val::number(1), val::number(1)]),
                ]),
            ]),
        ),
        (
            "let
                foo {x, y, 0}  = {^a, x, y}
                foo {_, 0, z}  = {^b, 0, z}
                foo {x, _, 42} = {^c, x}
                foo {x, y, z}  = {^d, x + y + z}
            in
                {foo {10, 20, 0}, foo {^ok, 0, 42.69}, foo {1, 2, 42}, foo {1, 2, 3}}",
            val::tuple([
                val::tuple([val::symbol("a"), val::number(10), val::number(20)]),
                val::tuple([val::symbol("b"), val::number(0), val::decimal_str("42.69")]),
                val::tuple([val::symbol("c"), val::number(1)]),
                val::tuple([val::symbol("d"), val::number(6)]),
            ]),
        ),
        (
            "let
                countMap {} = 0
                countMap {rest :: _ : _} = 1 + (countMap rest)
                map1 = {^foo: ^bar, ^bar: \"baz\"}
                map2 = {1: 2, 10: 20, ^hello: ^world}
            in
                {countMap {}, countMap map1, countMap map2}",
            val::tuple([val::number(0), val::number(2), val::number(3)]),
        ),
        (
            "let
                checkKeys {} = {}
                checkKeys {rest :: k : v} = {checkKeys rest :: k : {v, true}}

                map1 = {^foo: ^bar, ^bar: \"baz\"}
                map2 = {1: 2, 10: 20, ^hello: ^world}
            in
                {checkKeys {}, checkKeys map1, checkKeys map2}",
            val::tuple([
                val::map([]),
                val::map([
                    (
                        val::symbol("foo"),
                        val::tuple([val::symbol("bar"), val::bool(true)]),
                    ),
                    (
                        val::symbol("bar"),
                        val::tuple([val::string("baz"), val::bool(true)]),
                    ),
                ]),
                val::map([
                    (
                        val::number(1),
                        val::tuple([val::number(2), val::bool(true)]),
                    ),
                    (
                        val::number(10),
                        val::tuple([val::number(20), val::bool(true)]),
                    ),
                    (
                        val::symbol("hello"),
                        val::tuple([val::symbol("world"), val::bool(true)]),
                    ),
                ]),
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
