use crate::{EvalContext, Evaluate};
use rogato_common::val::{self};
use rogato_parser::{parse_expr, ParserContext};

#[test]
fn std_set_module() {
    let code_with_vals = [
        ("Set.from []", val::set([])),
        ("Set.from [1]", val::set([val::number(1)])),
        (
            "Set.from [1, ^foo]",
            val::set([val::number(1), val::symbol("foo")]),
        ),
        (
            "Set.from [1,2,1,3,2,1]",
            val::set([val::number(1), val::number(2), val::number(3)]),
        ),
        (
            "let
                s1 = Set.from [1, 2, 1, 3, 2, 1]
                s2 = Set.from [1, 2, 1, 3, 2, 1]
            in
                Set.merge s1 s2",
            val::set([val::number(1), val::number(2), val::number(3)]),
        ),
        (
            "let
                s1 = Set.from [1, 2, 1, 3, 2, 1]
                s2 = Set.from [1, 2, 4, 5, 6]
            in
                Set.merge s1 s2",
            val::set([
                val::number(1),
                val::number(2),
                val::number(3),
                val::number(4),
                val::number(5),
                val::number(6),
            ]),
        ),
        (
            "let
                s1 = Set.from [1, 2, 1, 3, 2, 1]
                s2 = Set.from [1, 2, 4, 5, 6]
                s3 = Set.merge s1 s2
            in
                s3 |> Set.map (val -> val * 2)",
            val::set([
                val::number(1 * 2),
                val::number(2 * 2),
                val::number(3 * 2),
                val::number(4 * 2),
                val::number(5 * 2),
                val::number(6 * 2),
            ]),
        ),
        (
            "let
                s1 = Set.from [1, 2, 1, 3, 2, 1]
                s2 = Set.from [1, 2, 4, 5, 6]
                s3 = Set.merge s1 s2
            in
                s3 |> Set.map (val -> {^value, val})",
            val::set([
                val::tuple([val::symbol("value"), val::number(1)]),
                val::tuple([val::symbol("value"), val::number(2)]),
                val::tuple([val::symbol("value"), val::number(3)]),
                val::tuple([val::symbol("value"), val::number(4)]),
                val::tuple([val::symbol("value"), val::number(5)]),
                val::tuple([val::symbol("value"), val::number(6)]),
            ]),
        ),
        (
            "let
                s1 = Set.from [1, 2, 1, 3, 2, 1]
                s2 = Set.from [1, 2, 4, 5, 6]
                s3 = Set.merge s1 s2
            in
                {Set.length s1, Set.length s2, Set.length s3}",
            val::tuple([val::number(3), val::number(5), val::number(6)]),
        ),
    ];

    let mut eval_ctx = EvalContext::new();
    let parser_ctx = ParserContext::new();

    for (code, val) in code_with_vals.iter() {
        let ast = parse_expr(code, &parser_ctx).unwrap();
        assert_eq!(ast.evaluate(&mut eval_ctx), Ok(val.clone()));
    }
}
