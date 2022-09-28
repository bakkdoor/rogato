#[cfg(test)]
use crate::{assert_parse, assert_parse_ast, assert_parse_expr, parse_expr, ParserContext};
#[cfg(test)]
use rogato_common::ast::helpers::inline_fn_def;
#[cfg(test)]
use rogato_common::ast::helpers::{
    commented, const_or_type_ref, db_type_ref, edge_prop, fn_call, fn_def, if_else, int_type,
    lambda, let_expr, list_lit, list_type, module_def, number_lit, op_call, program, prop_fn_ref,
    query, quoted, quoted_ast, root_comment, string_lit, string_type, struct_lit, struct_type,
    symbol, tuple_lit, tuple_type, type_def, type_ref, unquoted, unquoted_ast, var,
};
use rust_decimal_macros::dec;

#[test]
fn fn_defs() {
    assert_parse_ast!("let id x = x", fn_def("id", ["x"], var("x")));

    assert_parse_ast!(
        "let add a b = a + b",
        fn_def("add", ["a", "b"], op_call("+", var("a"), var("b")))
    );

    assert_parse_ast!(
        "let add a b c = (a + b) * (c * a)",
        fn_def(
            "add",
            ["a", "b", "c"],
            op_call(
                "*",
                op_call("+", var("a"), var("b")),
                op_call("*", var("c"), var("a"))
            )
        )
    );

    assert_parse_ast!(
        "let add1 a = 1 + a",
        fn_def("add1", ["a"], op_call("+", number_lit(1), var("a")))
    );

    assert_parse_ast!(
        "\nlet add1and2 = 1 + 2\n",
        fn_def("add1and2", [], op_call("+", number_lit(1), number_lit(2)))
    );

    assert_parse_ast!(
        "let foo a b = bar a (baz 1)",
        fn_def(
            "foo",
            ["a", "b"],
            fn_call("bar", [var("a"), fn_call("baz", [number_lit(1)])]),
        )
    );

    assert_parse_ast!(
        "let foo a b =
            let
                x = (
                    a + b
                )
                y = x *
                    a
                z = y * b
            in
                {x, y, z, {x, y}}
        ",
        fn_def(
            "foo",
            ["a", "b"],
            let_expr(
                [
                    ("x", op_call("+", var("a"), var("b"))),
                    ("y", op_call("*", var("x"), var("a"))),
                    ("z", op_call("*", var("y"), var("b"))),
                ],
                tuple_lit([
                    var("x"),
                    var("y"),
                    var("z"),
                    tuple_lit([var("x"), var("y")])
                ])
            )
        )
    );
}

#[test]
fn module_defs() {
    assert_parse_ast!("module MyModule", module_def("MyModule", []));

    assert_parse_ast!("module MyModule {}", module_def("MyModule", []));

    assert_parse_ast!("module MyModule {    }", module_def("MyModule", []));

    assert_parse_ast!("module MyModule {\n\n}", module_def("MyModule", []));

    assert_parse_ast!(
        "module MyModule {Foo_bar-baz}",
        module_def("MyModule", ["Foo_bar-baz"])
    );

    assert_parse_ast!(
        "module MyModule {foo, bar}",
        module_def("MyModule", ["foo", "bar"])
    );

    assert_parse_ast!(
        "module MyModule { func1, func2, func3 }",
        module_def("MyModule", ["func1", "func2", "func3"])
    );
}

#[test]
fn arithmetic_expressions() {
    assert_parse_expr!("1 + 1", op_call("+", number_lit(1), number_lit(1)));

    assert_parse_expr!("5 * 5", op_call("*", number_lit(5), number_lit(5)));

    assert_parse_expr!(
        "2 + (3 * 4)",
        op_call(
            "+",
            number_lit(2),
            op_call("*", number_lit(3), number_lit(4)),
        )
    );

    assert_parse_expr!(
        "(2 + 3) * 4",
        op_call(
            "*",
            op_call("+", number_lit(2), number_lit(3)),
            number_lit(4)
        )
    );

    assert_parse_expr!(
        "let x = 1, y = 2 in x + y",
        let_expr(
            [("x", number_lit(1)), ("y", number_lit(2))],
            op_call("+", var("x"), var("y")),
        )
    );

    let parser_ctx = ParserContext::new();

    assert!(parse_expr("(22+)+1", &parser_ctx).is_err());
    assert!(parse_expr("1++1", &parser_ctx).is_err());
    assert!(parse_expr("3)+1", &parser_ctx).is_err());
}

#[test]
fn literals() {
    assert_parse_expr!("0", number_lit(0));
    assert_parse_expr!("1", number_lit(1));
    assert_parse_expr!("-1", number_lit(-1));
    assert_parse_expr!("-10", number_lit(-10));
    assert_parse_expr!("10", number_lit(10));
    assert_parse_expr!("0.0", number_lit(dec!(0.0)));
    assert_parse_expr!("1.5", number_lit(dec!(1.5)));
    assert_parse_expr!("-1.2457", number_lit(dec!(-1.2457)));
    assert_parse_expr!("-10.8558", number_lit(dec!(-10.8558)));
    assert_parse_expr!("\"Hello, world!\"", string_lit("Hello, world!"));
    assert_parse_expr!(
        "{1,2,3}",
        tuple_lit([number_lit(1), number_lit(2), number_lit(3)])
    );

    assert_parse_expr!(
        "{ 1, (2 + 3), 4 }",
        tuple_lit([
            number_lit(1),
            op_call("+", number_lit(2), number_lit(3)),
            number_lit(4)
        ])
    );

    assert_parse_expr!(
        "{ 1, 2 + 3, 4 * 5 }",
        tuple_lit([
            number_lit(1),
            op_call("+", number_lit(2), number_lit(3)),
            op_call("*", number_lit(4), number_lit(5))
        ])
    );

    assert_parse_expr!(
        "{ 1, a + b, c * d }",
        tuple_lit([
            number_lit(1),
            op_call("+", var("a"), var("b")),
            op_call("*", var("c"), var("d"))
        ])
    );

    assert_parse_expr!(
        "{{x, x > y}, x == 0, x + y}",
        tuple_lit([
            tuple_lit([var("x"), op_call(">", var("x"), var("y"))]),
            op_call("==", var("x"), number_lit(0)),
            op_call("+", var("x"), var("y"))
        ])
    );

    assert_parse_expr!(
        "Person{id: 1, age: 35, country: \"Germany\"}",
        struct_lit(
            "Person",
            [
                ("id", number_lit(1)),
                ("age", number_lit(35)),
                ("country", string_lit("Germany"))
            ]
        )
    );

    assert_parse_expr!("[]", list_lit([]));

    assert_parse_expr!(
        "[
        ]",
        list_lit([])
    );

    assert_parse_expr!(
        "[
            // empty with comment
        ]",
        list_lit([])
    );

    assert_parse_expr!("[1]", list_lit([number_lit(1)]));

    assert_parse_expr!("[1, \"foo\"]", list_lit([number_lit(1), string_lit("foo")]));

    assert_parse_expr!(
        "[1, \"foo\", {2, \"bar\"}]",
        list_lit([
            number_lit(1),
            string_lit("foo"),
            tuple_lit([number_lit(2), string_lit("bar")])
        ])
    );

    assert_parse_expr!(
        "[[x, x > y], x == 0, x + y]",
        list_lit([
            list_lit([var("x"), op_call(">", var("x"), var("y"))]),
            op_call("==", var("x"), number_lit(0)),
            op_call("+", var("x"), var("y"))
        ])
    );
}

#[test]
fn fn_calls() {
    assert_parse_expr!("add 1 2", fn_call("add", [number_lit(1), number_lit(2)]));

    assert_parse_expr!("add 1 a", fn_call("add", [number_lit(1), var("a")]));

    assert_parse_expr!("add a 1", fn_call("add", [var("a"), number_lit(1)]));

    assert_parse_expr!(
        "add 1 (add 2 3)",
        fn_call(
            "add",
            [
                number_lit(1),
                fn_call("add", [number_lit(2), number_lit(3)]),
            ],
        )
    );

    assert_parse_expr!(
        "add 1 (add a 3)",
        fn_call(
            "add",
            [number_lit(1), fn_call("add", [var("a"), number_lit(3)]),],
        )
    );
}

#[test]
fn op_calls() {
    assert_parse_expr!("1 < 2", op_call("<", number_lit(1), number_lit(2)));

    assert_parse_expr!("1 > 2", op_call(">", number_lit(1), number_lit(2)));

    assert_parse_expr!("1 >> 2", op_call(">>", number_lit(1), number_lit(2)));

    assert_parse_expr!(
        "1 <= (2 + 3)",
        op_call(
            "<=",
            number_lit(1),
            op_call("+", number_lit(2), number_lit(3))
        )
    );

    assert_parse_expr!(
        "(2 + 3) <= foo",
        op_call("<=", op_call("+", number_lit(2), number_lit(3)), var("foo"))
    );

    assert_parse_expr!(
        "(2 + 3) <= (foo <!> (bar <=> baz))",
        op_call(
            "<=",
            op_call("+", number_lit(2), number_lit(3)),
            op_call("<!>", var("foo"), op_call("<=>", var("bar"), var("baz")))
        )
    );

    assert_parse_expr!(
        "(1 >> 3) != 2",
        op_call(
            "!=",
            op_call(">>", number_lit(1), number_lit(3)),
            number_lit(2)
        )
    );

    assert_parse_expr!(
        "(foo bar 1) != 2",
        op_call(
            "!=",
            fn_call("foo", [var("bar"), number_lit(1)]),
            number_lit(2)
        )
    );

    assert_parse_expr!(
        "1 >> 2 >> 5 >> (a << b << (c >> d))",
        op_call(
            ">>",
            op_call(
                ">>",
                op_call(">>", number_lit(1), number_lit(2)),
                number_lit(5)
            ),
            op_call(
                "<<",
                op_call("<<", var("a"), var("b")),
                op_call(">>", var("c"), var("d"))
            )
        )
    );

    assert_parse_expr!(
        "1 <- 2 <- 3",
        op_call(
            "<-",
            op_call("<-", number_lit(1), number_lit(2)),
            number_lit(3),
        )
    );

    assert_parse_expr!(
        "1 <-
         2 <-
         3",
        op_call(
            "<-",
            op_call("<-", number_lit(1), number_lit(2)),
            number_lit(3),
        )
    );
}

#[test]
fn comments() {
    assert_parse!("// a comment", program([root_comment(" a comment")]));

    assert_parse!(
        "// a comment\n       // another comment",
        program([root_comment(" a comment"), root_comment(" another comment")])
    );

    assert_parse!(
        "// a comment\n\t // \n\n\t //what ok         \n       // another comment",
        program([
            root_comment(" a comment"),
            root_comment(" "),
            root_comment("what ok         "),
            root_comment(" another comment")
        ])
    );

    assert_parse_expr!(
        "// a comment yo!\nlet x = 1 in x * 2",
        commented(
            " a comment yo!",
            let_expr(
                [("x", number_lit(1))],
                op_call("*", var("x"), number_lit(2))
            )
        )
    );
}

#[test]
fn type_defs() {
    let person_type_def = type_def(
        "Person",
        struct_type([
            ("name", string_type()),
            ("age", int_type()),
            ("country", type_ref("Country")),
        ]),
    );

    assert_parse_ast!(
        "type Person :: {
            name :: String
            age :: Int
            country :: Country
        }",
        person_type_def.to_owned()
    );

    assert_parse_ast!(
        "type Person :: {
            name :: String, age :: Int, country :: Country
        }",
        person_type_def.to_owned()
    );

    assert_parse_ast!(
        "type Person :: {
            name :: String,     age :: Int,     country :: Country
        }",
        person_type_def.to_owned()
    );

    assert_parse_ast!(
        "type Person :: {
            name :: String,     age :: Int
                 country :: Country
        }",
        person_type_def.to_owned()
    );

    assert_parse_ast!(
        "type Pair :: {A, B}",
        type_def("Pair", tuple_type([type_ref("A"), type_ref("B")]))
    );

    assert_parse_ast!(
        "type @People :: [Person]",
        type_def("@People", list_type(type_ref("Person")))
    );
}

#[test]
fn let_expressions() {
    assert_parse_expr!(
        "let name = \"John Connor\"
             age = 12
             city = \"Los Angeles\"
        in
            {name, age, city}",
        let_expr(
            [
                ("name", string_lit("John Connor")),
                ("age", number_lit(12)),
                ("city", string_lit("Los Angeles"))
            ],
            tuple_lit([var("name"), var("age"), var("city")])
        )
    );

    assert_parse_expr!(
        "let name = \"John Connor\",
             age = 12,
             city = \"Los Angeles\"
        in
            Person{name: name, age: age, city: city}",
        let_expr(
            [
                ("name", string_lit("John Connor")),
                ("age", number_lit(12)),
                ("city", string_lit("Los Angeles"))
            ],
            struct_lit(
                "Person",
                [
                    ("name", var("name")),
                    ("age", var("age")),
                    ("city", var("city"))
                ]
            )
        )
    );

    assert_parse_expr!(
        "let
            friendsOfFriends =
                ? f1, f2 <- people
                ? f1 <- friends person
                ? f2 <- friends f1
                !> f2,
            friendsNames =
                (List.map friendsOfFriends name)
         in
            { friends, List.count friends }",
        let_expr(
            [
                (
                    "friendsOfFriends",
                    query(
                        [
                            (vec!["f1", "f2"], var("people"), false),
                            (vec!["f1"], fn_call("friends", [var("person")]), false),
                            (vec!["f2"], fn_call("friends", [var("f1")]), false)
                        ],
                        [],
                        var("f2")
                    )
                ),
                (
                    "friendsNames",
                    fn_call("List.map", [var("friendsOfFriends"), var("name")])
                )
            ],
            tuple_lit([var("friends"), fn_call("List.count", [var("friends")])])
        )
    );

    assert_parse_expr!(
        "let
            add a b = a + b
            mul a b =
                a * b
         in
            { add 1 2, mul 2 3 }",
        let_expr(
            [
                (
                    "add",
                    inline_fn_def("add", ["a", "b"], op_call("+", var("a"), var("b")))
                ),
                (
                    "mul",
                    inline_fn_def("mul", ["a", "b"], op_call("*", var("a"), var("b")))
                ),
            ],
            tuple_lit([
                fn_call("add", [number_lit(1), number_lit(2)]),
                fn_call("mul", [number_lit(2), number_lit(3)]),
            ])
        )
    );
}

#[test]
fn queries() {
    assert_parse_expr!(
        "? p <- Person
        ! isOlderThan p 42
        ! isPopular p
        !> p",
        query(
            [(["p"], const_or_type_ref("Person"), false)],
            [
                fn_call("isOlderThan", [var("p"), number_lit(42)]),
                fn_call("isPopular", [var("p")])
            ],
            var("p")
        )
    );

    assert_parse_expr!(
        "? p <- Person
        ? p2 <- Person
        ! isOlderThan p 42
        ! isPopular p
        ! isFriendOf p p2
        !> {p, p2}",
        query(
            [
                (["p"], const_or_type_ref("Person"), false),
                (["p2"], const_or_type_ref("Person"), false)
            ],
            [
                fn_call("isOlderThan", [var("p"), number_lit(42)]),
                fn_call("isPopular", [var("p")]),
                fn_call("isFriendOf", [var("p"), var("p2")])
            ],
            tuple_lit([var("p"), var("p2")])
        )
    );

    assert_parse_expr!(
        "? p <- Person
        ? p2 <- Person
        ! ((age p) == ((age p2) + 1))
        !> {p, p2}",
        query(
            [
                (["p"], const_or_type_ref("Person"), false),
                (["p2"], const_or_type_ref("Person"), false)
            ],
            [op_call(
                "==",
                fn_call("age", [var("p")]),
                op_call("+", fn_call("age", [var("p2")]), number_lit(1))
            ),],
            tuple_lit([var("p"), var("p2")])
        )
    );

    assert_parse_expr!(
        "? p1 <- people
         ? p2 <- people
         ? f <- friends p1
         ? f <- friends p2
         !> f",
        query(
            [
                (["p1"], var("people"), false),
                (["p2"], var("people"), false),
                (["f"], fn_call("friends", [var("p1")]), false),
                (["f"], fn_call("friends", [var("p2")]), false),
            ],
            [],
            var("f"),
        )
    );

    assert_parse_expr!(
        "? p1, p2 <- people
         ? f <- friends p1
         ? f <- friends p2
         !> f",
        query(
            [
                (vec!["p1", "p2"], var("people"), false),
                (vec!["f"], fn_call("friends", [var("p1")]), false),
                (vec!["f"], fn_call("friends", [var("p2")]), false),
            ],
            [],
            var("f"),
        )
    );

    assert_parse_expr!(
        "? a,
           b
           ,c <- Data
         ? diff <- dataDiff a b c
         ! 0 < (a + (b + c))
         !> {diff, {a, b, c}}",
        query(
            [
                (vec!["a", "b", "c"], const_or_type_ref("Data"), false),
                (
                    vec!["diff"],
                    fn_call("dataDiff", [var("a"), var("b"), var("c")]),
                    false
                )
            ],
            [op_call(
                "<",
                number_lit(0),
                op_call("+", var("a"), op_call("+", var("b"), var("c")))
            )],
            tuple_lit([var("diff"), tuple_lit([var("a"), var("b"), var("c")])]),
        )
    );

    assert_parse_expr!(
        "? bob <- Person
         ? joe <- Person
         ? bob <- job#Friend
         !> {bob, joe}",
        query(
            [
                (["bob"], const_or_type_ref("Person"), false),
                (["joe"], const_or_type_ref("Person"), false),
                (["bob"], edge_prop(var("job"), "Friend"), false)
            ],
            [],
            tuple_lit([var("bob"), var("joe")])
        )
    );

    assert_parse_expr!(
        "? p <- Person
         ? p <!- oldPeople
         !> p",
        query(
            [
                (["p"], const_or_type_ref("Person"), false),
                (["p"], var("oldPeople"), true)
            ],
            [],
            var("p")
        )
    );

    assert_parse_expr!(
        "? p <- Person
         ? p <- hello 1 2
         ? p <!- (a | b)#Friend
         !> inspect p",
        query(
            [
                (["p"], const_or_type_ref("Person"), false),
                (
                    ["p"],
                    fn_call("hello", [number_lit(1), number_lit(2)]),
                    false
                ),
                (
                    ["p"],
                    edge_prop(op_call("|", var("a"), var("b")), "Friend"),
                    true
                )
            ],
            [],
            fn_call("inspect", [var("p")])
        )
    );

    assert_parse_expr!(
        "? p <- Person
         ? p <- hello 1 2
         ? p <!- (
            ? p2 <- (a | b)
            !> p2
         )#Friend
         ? p <!- (
            p
            |> friendsWithoutFriends
         )
         !> inspect p",
        query(
            [
                (["p"], const_or_type_ref("Person"), false),
                (
                    ["p"],
                    fn_call("hello", [number_lit(1), number_lit(2)]),
                    false
                ),
                (
                    ["p"],
                    edge_prop(
                        query(
                            [(["p2"], op_call("|", var("a"), var("b")), false)],
                            [],
                            var("p2")
                        ),
                        "Friend"
                    ),
                    true
                ),
                (["p"], fn_call("friendsWithoutFriends", [var("p")]), true)
            ],
            [],
            fn_call("inspect", [var("p")])
        )
    );
}

#[test]
fn variables() {
    assert_parse_expr!("foo", var("foo"));

    assert_parse_expr!(".foo", prop_fn_ref("foo"));

    assert_parse_expr!("Foo", const_or_type_ref("Foo"));

    assert_parse_expr!(
        "{.foo, .bar}",
        tuple_lit([prop_fn_ref("foo"), prop_fn_ref("bar")])
    );

    assert_parse_expr!("@People", db_type_ref("People"));

    parse_expr(
        "? p <- @People
         ! (hasFriends p)
         !> {p.name, p}
        ",
        &mut ParserContext::new(),
    )
    .unwrap();
}

#[test]
fn lambdas() {
    assert_parse_expr!(
        "x -> x + 1",
        lambda(["x"], op_call("+", var("x"), number_lit(1)))
    );

    assert_parse_expr!(
        "x y -> x + y",
        lambda(["x", "y"], op_call("+", var("x"), var("y")))
    );

    assert_parse_expr!(
        "x y -> inspect (z -> {x * (y * z), {x, y, z}}) x y",
        lambda(
            ["x", "y"],
            fn_call(
                "inspect",
                [
                    lambda(
                        ["z"],
                        tuple_lit([
                            op_call("*", var("x"), op_call("*", var("y"), var("z"))),
                            tuple_lit([var("x"), var("y"), var("z")])
                        ])
                    ),
                    var("x"),
                    var("y")
                ]
            )
        )
    );

    assert_parse_expr!(
        "w x -> y -> z -> {w, x, y, z}",
        lambda(
            ["w", "x"],
            lambda(
                ["y"],
                lambda(["z"], tuple_lit([var("w"), var("x"), var("y"), var("z")]))
            )
        )
    );
}

#[test]
fn fn_pipes() {
    assert_parse_expr!(
        "2 |> inspect |> MyMod.doStuff |> repeat",
        fn_call(
            "repeat",
            [fn_call(
                "MyMod.doStuff",
                [fn_call("inspect", [number_lit(2)])]
            )]
        )
    );

    assert_parse_expr!(
        "2 |> inspect |> repeat 10",
        fn_call(
            "repeat",
            [fn_call("inspect", [number_lit(2)]), number_lit(10)]
        )
    );

    assert_parse_expr!(
        "[1, 2]
         |> joinWith {1,2,3}
         |> select (x -> isHappy x)
         |> sumBy (delta 1)",
        fn_call(
            "sumBy",
            [
                fn_call(
                    "select",
                    [
                        fn_call(
                            "joinWith",
                            [
                                list_lit([number_lit(1), number_lit(2)]),
                                tuple_lit([number_lit(1), number_lit(2), number_lit(3)])
                            ]
                        ),
                        lambda(["x"], fn_call("isHappy", [var("x")]))
                    ]
                ),
                fn_call("delta", [number_lit(1)])
            ]
        )
    );

    assert_parse_ast!(
        "let f x =
            ? p <- Person
            ! (x
                |> doStuff {2, 3, 4}
                |> thenDo (y -> x + (y |> toString |> join {1,2,3}))
            )
            !> p",
        fn_def(
            "f",
            ["x"],
            query(
                [(["p"], const_or_type_ref("Person"), false)],
                [fn_call(
                    "thenDo",
                    [
                        fn_call(
                            "doStuff",
                            [
                                var("x"),
                                tuple_lit([number_lit(2), number_lit(3), number_lit(4)])
                            ]
                        ),
                        lambda(
                            ["y"],
                            op_call(
                                "+",
                                var("x"),
                                fn_call(
                                    "join",
                                    [
                                        fn_call("toString", [var("y")],),
                                        tuple_lit([number_lit(1), number_lit(2), number_lit(3)])
                                    ]
                                )
                            )
                        )
                    ]
                )],
                var("p")
            )
        )
    );
}

#[test]
fn symbols_and_quotes() {
    assert_parse_expr!("^symbol", symbol("symbol"));

    assert_parse_expr!("^AnotherSymbol", symbol("AnotherSymbol"));

    assert_parse_expr!(
        "^(quoted expression)",
        quoted(fn_call("quoted", [var("expression")]))
    );

    assert_parse_expr!(
        "^(quoted (expression \"in quotes\" {1, 2, 3}))",
        quoted(fn_call(
            "quoted",
            [fn_call(
                "expression",
                [
                    string_lit("in quotes"),
                    tuple_lit([number_lit(1), number_lit(2), number_lit(3)])
                ]
            )]
        ))
    );

    assert_parse_expr!(
        "^(quoted ~var in ~var2)",
        quoted(fn_call(
            "quoted",
            [unquoted(var("var")), var("in"), unquoted(var("var2"))]
        ))
    );

    assert_parse_expr!(
        "^(quoted ~(var {~var2, 123, ~var3, ^Cool}))",
        quoted(fn_call(
            "quoted",
            [unquoted(fn_call(
                "var",
                [tuple_lit([
                    unquoted(var("var2")),
                    number_lit(123),
                    unquoted(var("var3")),
                    symbol("Cool")
                ])]
            ))]
        ))
    );

    assert_parse_expr!(
        "^(quoted ^Cool)",
        quoted(fn_call("quoted", [symbol("Cool")]))
    );

    assert_parse_expr!(
        "^(let f x = x + 1)",
        quoted_ast(fn_def("f", ["x"], op_call("+", var("x"), number_lit(1))))
    );

    assert_parse_expr!(
        "~(let f x = x + 1)",
        unquoted_ast(fn_def("f", ["x"], op_call("+", var("x"), number_lit(1))))
    );
}

#[test]
fn if_else_expr() {
    assert_parse_expr!(
        "if (x > y) then x else y",
        if_else(op_call(">", var("x"), var("y")), var("x"), var("y"))
    );

    assert_parse_expr!("if x then x else y", if_else(var("x"), var("x"), var("y")));

    assert_parse_ast!(
        "let f x y = if (x > y) then {x, y} else {y, x}",
        fn_def(
            "f",
            ["x", "y"],
            if_else(
                op_call(">", var("x"), var("y")),
                tuple_lit([var("x"), var("y")]),
                tuple_lit([var("y"), var("x")])
            )
        )
    );
}
