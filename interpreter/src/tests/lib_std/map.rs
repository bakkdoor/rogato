use crate::{tests::parse_eval_std, EvalContext, Evaluate};
use rogato_common::val::{self};
use rogato_parser::{parse_expr, ParserContext};

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
                    map |> Map.remove ^foo,
                    map |> Map.remove ^bar,
                    map |> Map.remove {1, 2},
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
        (
            "let
                map1 = {^foo: 1, ^bar: {1, 2, 3}}
                map2 = {^foo: 2, {1, 2}: ^wat}
                map3 = {^foo: 3, ^wat: {420, 69}}
            in
                {
                    map1 |> Map.merge map2,
                    map2 |> Map.merge map3,
                    map1 |> Map.merge map2 |> Map.merge map3
                }",
            val::tuple([
                val::map([
                    (val::symbol("foo"), val::number(2)),
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
                    (val::symbol("foo"), val::number(3)),
                    (
                        val::tuple([val::number(1), val::number(2)]),
                        val::symbol("wat"),
                    ),
                    (
                        val::symbol("wat"),
                        val::tuple([val::number(420), val::number(69)]),
                    ),
                ]),
                val::map([
                    (val::symbol("foo"), val::number(3)),
                    (
                        val::symbol("bar"),
                        val::tuple([val::number(1), val::number(2), val::number(3)]),
                    ),
                    (
                        val::tuple([val::number(1), val::number(2)]),
                        val::symbol("wat"),
                    ),
                    (
                        val::symbol("wat"),
                        val::tuple([val::number(420), val::number(69)]),
                    ),
                ]),
            ]),
        ),
        (
            "let
                map = {^foo: 1, ^bar: {1, 2, 3}, {1, 2}: ^wat}
            in
                {
                    map |> Map.getOrElse ^foo 2,
                    map |> Map.getOrElse ^baz 2,
                    map |> Map.getOrElse {1, 2} 2,
                    map |> Map.getOrElse {1, 3} 2,
                }",
            val::tuple([
                val::number(1),
                val::number(2),
                val::symbol("wat"),
                val::number(2),
            ]),
        ),
        (
            "let
                map = {^foo: 1, ^bar: {1, 2, 3}, {1, 2}: ^wat}
            in
                map |> Map.filter (k v -> k == ^foo)",
            val::map([(val::symbol("foo"), val::number(1))]),
        ),
        (
            "let
                map = {^foo: 1, ^bar: {1, 2, 3}, {1, 2}: ^wat}
            in
                map |> Map.filter (k v -> v == {1, 2, 3})",
            val::map([(
                val::symbol("bar"),
                val::tuple([val::number(1), val::number(2), val::number(3)]),
            )]),
        ),
        (
            "let
                map = {^foo: 1, ^bar: {1, 2, 3}, {1, 2}: ^wat}
            in
                map |> Map.filter (k v -> (k == ^foo) || (v == {1, 2, 3}))",
            val::map([
                (val::symbol("foo"), val::number(1)),
                (
                    val::symbol("bar"),
                    val::tuple([val::number(1), val::number(2), val::number(3)]),
                ),
            ]),
        ),
    ];

    let mut eval_ctx = EvalContext::new();
    let parser_ctx = ParserContext::new();

    parse_eval_std("Map", &parser_ctx, &mut eval_ctx);

    for (code, val) in code_with_vals.iter() {
        let ast = parse_expr(code, &parser_ctx).unwrap();
        assert_eq!(ast.evaluate(&mut eval_ctx), Ok(val.clone()));
    }
}
