use crate::{tests::parse_eval_std, EvalContext, Evaluate};
use rogato_common::val::{self};
use rogato_parser::{parse_expr, ParserContext};

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
                    List.map [1,2,3] insp,
                    List.map [10,20,30] ^inspect,
                    List.map [10,20,30] add10,
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
        (
            "List.zip4 (range 0 3) (range 10 13) (range 100 103) (range 1000 1003)",
            val::list([
                val::tuple([
                    val::number(0),
                    val::number(10),
                    val::number(100),
                    val::number(1000),
                ]),
                val::tuple([
                    val::number(1),
                    val::number(11),
                    val::number(101),
                    val::number(1001),
                ]),
                val::tuple([
                    val::number(2),
                    val::number(12),
                    val::number(102),
                    val::number(1002),
                ]),
            ]),
        ),
        (
            "List.zipWith (range 5) (range 5 10) (x y -> {^x: x, ^y: y})",
            val::list([
                val::map([
                    (val::symbol("x"), val::number(0)),
                    (val::symbol("y"), val::number(5)),
                ]),
                val::map([
                    (val::symbol("x"), val::number(1)),
                    (val::symbol("y"), val::number(6)),
                ]),
                val::map([
                    (val::symbol("x"), val::number(2)),
                    (val::symbol("y"), val::number(7)),
                ]),
                val::map([
                    (val::symbol("x"), val::number(3)),
                    (val::symbol("y"), val::number(8)),
                ]),
                val::map([
                    (val::symbol("x"), val::number(4)),
                    (val::symbol("y"), val::number(9)),
                ]),
            ]),
        ),
        (
            "List.countBy (range 5) (x -> x + 1)",
            val::number(1 + 2 + 3 + 4 + 5),
        ),
        (
            "List.countBy [range 5, range 10, range 100] ^length",
            val::number(5 + 10 + 100),
        ),
    ];

    let mut eval_ctx = EvalContext::new();
    let parser_ctx = ParserContext::new();

    parse_eval_std("List", &parser_ctx, &mut eval_ctx);

    for (code, val) in code_with_vals.iter() {
        let ast = parse_expr(code, &parser_ctx).unwrap();
        assert_eq!(ast.evaluate(&mut eval_ctx), Ok(val.clone()));
    }
}
