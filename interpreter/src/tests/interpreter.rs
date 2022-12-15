use crate::{EvalContext, Evaluate};
use rogato_common::val::{self};
use rogato_parser::{parse_expr, ParserContext};

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
            list = [10,100,1000] |> Std.List.map ^g
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
        (
            "let
                count list = count_ 0 list
                count_ count [] = count
                count_ count [_ :: rest] = count_ (1 + count) rest
            in
                {count (range 10), count (range 10000), count (range 25000)}",
            val::tuple([val::number(10), val::number(10000), val::number(25000)]),
        ),
        (
            "(range 5) |> Std.List.map (0 -> ^done, n -> {^got, n})",
            val::list([
                val::symbol("done"),
                val::tuple([val::symbol("got"), val::number(1)]),
                val::tuple([val::symbol("got"), val::number(2)]),
                val::tuple([val::symbol("got"), val::number(3)]),
                val::tuple([val::symbol("got"), val::number(4)]),
            ]),
        ),
        (
            "(range 5) |> Std.List.reverse |> Std.List.map (0 -> ^done, n -> {^got, n})",
            val::list([
                val::tuple([val::symbol("got"), val::number(4)]),
                val::tuple([val::symbol("got"), val::number(3)]),
                val::tuple([val::symbol("got"), val::number(2)]),
                val::tuple([val::symbol("got"), val::number(1)]),
                val::symbol("done"),
            ]),
        ),
        (
            "[1,0,1,0,1,1,0,0] |> Std.List.map (0 -> ^zero, 1 -> ^one)",
            val::list([
                val::symbol("one"),
                val::symbol("zero"),
                val::symbol("one"),
                val::symbol("zero"),
                val::symbol("one"),
                val::symbol("one"),
                val::symbol("zero"),
                val::symbol("zero"),
            ]),
        ),
        (
            "[[], [1,2,3], [1]] |> Std.List.map (
              [] -> ^empty,
              [n] -> {n,n,n},
              [_ :: xs] -> {^rest, xs}
            )",
            val::list([
                val::symbol("empty"),
                val::tuple([
                    val::symbol("rest"),
                    val::list([val::number(2), val::number(3)]),
                ]),
                val::tuple([val::number(1), val::number(1), val::number(1)]),
            ]),
        ),
        (
            "let
                f [] = ^empty
                f [x] = {^single, x}
                f [a,b] = {^pair, {a, b}}
                f [a,b,c] = {^triplet, {a,b,c}}
                f list = {^list, list}
            in
                {
                    match (range 0) ^f,
                    match (range 1) ^f,
                    match (range 2) ^f,
                    match (range 3) ^f,
                    match (range 4) ^f,
                    match (range 5) ^f,
                    match (range 10) ^f
                }",
            val::tuple([
                val::symbol("empty"),
                val::tuple([val::symbol("single"), val::number(0)]),
                val::tuple([
                    val::symbol("pair"),
                    val::tuple([val::number(0), val::number(1)]),
                ]),
                val::tuple([
                    val::symbol("triplet"),
                    val::tuple([val::number(0), val::number(1), val::number(2)]),
                ]),
                val::tuple([
                    val::symbol("list"),
                    val::list([
                        val::number(0),
                        val::number(1),
                        val::number(2),
                        val::number(3),
                    ]),
                ]),
                val::tuple([
                    val::symbol("list"),
                    val::list([
                        val::number(0),
                        val::number(1),
                        val::number(2),
                        val::number(3),
                        val::number(4),
                    ]),
                ]),
                val::tuple([
                    val::symbol("list"),
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
