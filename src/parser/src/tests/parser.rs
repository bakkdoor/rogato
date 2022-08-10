#[cfg(test)]
use crate::{assert_parse, assert_parse_ast, assert_parse_expr};

#[cfg(test)]
use rogato_common::ast::helpers::{
    commented, const_or_type_ref, edge_prop, fn_call, fn_def, int_lit, int_type, lambda, let_expr,
    list_lit, module_def, op_call, program, prop_fn_ref, query, quoted, quoted_ast, root_comment,
    string_lit, string_type, struct_lit, struct_type, symbol, tuple_lit, tuple_type, type_def,
    type_ref, unquoted, unquoted_ast, var,
};

#[cfg(test)]
use crate::parse_expr;

#[test]
fn fn_defs() {
    assert_parse_ast!("let id x = x", fn_def("id", vec!["x"], var("x")));

    assert_parse_ast!(
        "let add a b = a + b",
        fn_def("add", vec!["a", "b"], op_call("+", var("a"), var("b")))
    );

    assert_parse_ast!(
        "let add a b c = (a + b) * (c * a)",
        fn_def(
            "add",
            vec!["a", "b", "c"],
            op_call(
                "*",
                op_call("+", var("a"), var("b")),
                op_call("*", var("c"), var("a"))
            )
        )
    );

    assert_parse_ast!(
        "let add1 a = 1 + a",
        fn_def("add1", vec!["a"], op_call("+", int_lit(1), var("a")))
    );

    assert_parse_ast!(
        "\nlet add1and2 = 1 + 2\n",
        fn_def("add1and2", vec![], op_call("+", int_lit(1), int_lit(2)))
    );

    assert_parse_ast!(
        "let foo a b = bar a (baz 1)",
        fn_def(
            "foo",
            vec!["a", "b"],
            fn_call("bar", vec![var("a"), fn_call("baz", vec![int_lit(1)])]),
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
            vec!["a", "b"],
            let_expr(
                vec![
                    ("x", op_call("+", var("a"), var("b"))),
                    ("y", op_call("*", var("x"), var("a"))),
                    ("z", op_call("*", var("y"), var("b"))),
                ],
                tuple_lit(vec![
                    var("x"),
                    var("y"),
                    var("z"),
                    tuple_lit(vec![var("x"), var("y")])
                ])
            )
        )
    );
}

#[test]
fn module_defs() {
    assert_parse_ast!("module MyModule", module_def("MyModule", vec![]));

    assert_parse_ast!("module MyModule {}", module_def("MyModule", vec![]));

    assert_parse_ast!("module MyModule {    }", module_def("MyModule", vec![]));

    assert_parse_ast!("module MyModule {\n\n}", module_def("MyModule", vec![]));

    assert_parse_ast!(
        "module MyModule {Foo_bar-baz}",
        module_def("MyModule", vec!["Foo_bar-baz"])
    );

    assert_parse_ast!(
        "module MyModule {foo, bar}",
        module_def("MyModule", vec!["foo", "bar"])
    );

    assert_parse_ast!(
        "module MyModule { func1, func2, func3 }",
        module_def("MyModule", vec!["func1", "func2", "func3"])
    );
}

#[test]
fn arithmetic_expressions() {
    assert_parse_expr!("1 + 1", op_call("+", int_lit(1), int_lit(1)));

    assert_parse_expr!("5 * 5", op_call("*", int_lit(5), int_lit(5)));

    assert_parse_expr!(
        "2 + (3 * 4)",
        op_call("+", int_lit(2), op_call("*", int_lit(3), int_lit(4)),)
    );

    assert_parse_expr!(
        "(2 + 3) * 4",
        op_call("*", op_call("+", int_lit(2), int_lit(3)), int_lit(4))
    );

    assert_parse_expr!(
        "let x = 1, y = 2 in x + y",
        let_expr(
            vec![("x", int_lit(1)), ("y", int_lit(2))],
            op_call("+", var("x"), var("y")),
        )
    );

    assert!(parse_expr("(22+)+1").is_err());

    assert!(parse_expr("1++1").is_err());

    assert!(parse_expr("3)+1").is_err());
}

#[test]
fn literals() {
    assert_parse_expr!("1", int_lit(1));

    assert_parse_expr!("\"Hello, world!\"", string_lit("Hello, world!"));

    assert_parse_expr!(
        "{1,2,3}",
        tuple_lit(vec![int_lit(1), int_lit(2), int_lit(3)])
    );

    assert_parse_expr!(
        "{ 1, (2 + 3), 4 }",
        tuple_lit(vec![
            int_lit(1),
            op_call("+", int_lit(2), int_lit(3)),
            int_lit(4)
        ])
    );

    assert_parse_expr!(
        "{ 1, 2 + 3, 4 * 5 }",
        tuple_lit(vec![
            int_lit(1),
            op_call("+", int_lit(2), int_lit(3)),
            op_call("*", int_lit(4), int_lit(5))
        ])
    );

    assert_parse_expr!(
        "{ 1, a + b, c * d }",
        tuple_lit(vec![
            int_lit(1),
            op_call("+", var("a"), var("b")),
            op_call("*", var("c"), var("d"))
        ])
    );

    assert_parse_expr!(
        "{{x, x > y}, x == 0, x + y}",
        tuple_lit(vec![
            tuple_lit(vec![var("x"), op_call(">", var("x"), var("y"))]),
            op_call("==", var("x"), int_lit(0)),
            op_call("+", var("x"), var("y"))
        ])
    );

    assert_parse_expr!(
        "Person{id: 1, age: 35, country: \"Germany\"}",
        struct_lit(
            "Person",
            vec![
                ("id", int_lit(1)),
                ("age", int_lit(35)),
                ("country", string_lit("Germany"))
            ]
        )
    );

    assert_parse_expr!("[]", list_lit(vec![]));

    assert_parse_expr!(
        "[
        ]",
        list_lit(vec![])
    );

    assert_parse_expr!(
        "[
            // empty with comment
        ]",
        list_lit(vec![])
    );

    assert_parse_expr!("[1]", list_lit(vec![int_lit(1)]));

    assert_parse_expr!(
        "[1, \"foo\"]",
        list_lit(vec![int_lit(1), string_lit("foo")])
    );

    assert_parse_expr!(
        "[1, \"foo\", {2, \"bar\"}]",
        list_lit(vec![
            int_lit(1),
            string_lit("foo"),
            tuple_lit(vec![int_lit(2), string_lit("bar")])
        ])
    );

    assert_parse_expr!(
        "[[x, x > y], x == 0, x + y]",
        list_lit(vec![
            list_lit(vec![var("x"), op_call(">", var("x"), var("y"))]),
            op_call("==", var("x"), int_lit(0)),
            op_call("+", var("x"), var("y"))
        ])
    );
}

#[test]
fn fn_calls() {
    assert_parse_expr!("add 1 2", fn_call("add", vec![int_lit(1), int_lit(2)]));

    assert_parse_expr!("add 1 a", fn_call("add", vec![int_lit(1), var("a")]));

    assert_parse_expr!("add a 1", fn_call("add", vec![var("a"), int_lit(1)]));

    assert_parse_expr!(
        "add 1 (add 2 3)",
        fn_call(
            "add",
            vec![int_lit(1), fn_call("add", vec![int_lit(2), int_lit(3)]),],
        )
    );

    assert_parse_expr!(
        "add 1 (add a 3)",
        fn_call(
            "add",
            vec![int_lit(1), fn_call("add", vec![var("a"), int_lit(3)]),],
        )
    );
}

#[test]
fn op_calls() {
    assert_parse_expr!("1 < 2", op_call("<", int_lit(1), int_lit(2)));

    assert_parse_expr!("1 > 2", op_call(">", int_lit(1), int_lit(2)));

    assert_parse_expr!("1 >> 2", op_call(">>", int_lit(1), int_lit(2)));

    assert_parse_expr!(
        "1 <= (2 + 3)",
        op_call("<=", int_lit(1), op_call("+", int_lit(2), int_lit(3)))
    );

    assert_parse_expr!(
        "(2 + 3) <= foo",
        op_call("<=", op_call("+", int_lit(2), int_lit(3)), var("foo"))
    );

    assert_parse_expr!(
        "(2 + 3) <= (foo <!> (bar <=> baz))",
        op_call(
            "<=",
            op_call("+", int_lit(2), int_lit(3)),
            op_call("<!>", var("foo"), op_call("<=>", var("bar"), var("baz")))
        )
    );

    assert_parse_expr!(
        "(1 >> 3) != 2",
        op_call("!=", op_call(">>", int_lit(1), int_lit(3)), int_lit(2))
    );

    assert_parse_expr!(
        "(foo bar 1) != 2",
        op_call(
            "!=",
            fn_call("foo", vec![var("bar"), int_lit(1)]),
            int_lit(2)
        )
    );
}

#[test]
fn comments() {
    assert_parse!("// a comment", program(vec![root_comment(" a comment")]));

    assert_parse!(
        "// a comment\n       // another comment",
        program(vec![
            root_comment(" a comment"),
            root_comment(" another comment")
        ])
    );

    assert_parse!(
        "// a comment\n\t // \n\n\t //what ok         \n       // another comment",
        program(vec![
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
            let_expr(vec![("x", int_lit(1))], op_call("*", var("x"), int_lit(2)))
        )
    );
}

#[test]
fn type_defs() {
    let person_type_def = type_def(
        "Person",
        struct_type(vec![
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
        type_def("Pair", tuple_type(vec![type_ref("A"), type_ref("B")]))
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
            vec![
                ("name", string_lit("John Connor")),
                ("age", int_lit(12)),
                ("city", string_lit("Los Angeles"))
            ],
            tuple_lit(vec![var("name"), var("age"), var("city")])
        )
    );

    assert_parse_expr!(
        "let name = \"John Connor\",
             age = 12,
             city = \"Los Angeles\"
        in
            Person{name: name, age: age, city: city}",
        let_expr(
            vec![
                ("name", string_lit("John Connor")),
                ("age", int_lit(12)),
                ("city", string_lit("Los Angeles"))
            ],
            struct_lit(
                "Person",
                vec![
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
            vec![
                (
                    "friendsOfFriends",
                    query(
                        vec![
                            (vec!["f1", "f2"], var("people"), false),
                            (vec!["f1"], fn_call("friends", vec![var("person")]), false),
                            (vec!["f2"], fn_call("friends", vec![var("f1")]), false)
                        ],
                        vec![],
                        var("f2")
                    )
                ),
                (
                    "friendsNames",
                    fn_call("List.map", vec![var("friendsOfFriends"), var("name")])
                )
            ],
            tuple_lit(vec![
                var("friends"),
                fn_call("List.count", vec![var("friends")])
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
            vec![(vec!["p"], const_or_type_ref("Person"), false)],
            vec![
                fn_call("isOlderThan", vec![var("p"), int_lit(42)]),
                fn_call("isPopular", vec![var("p")])
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
            vec![
                (vec!["p"], const_or_type_ref("Person"), false),
                (vec!["p2"], const_or_type_ref("Person"), false)
            ],
            vec![
                fn_call("isOlderThan", vec![var("p"), int_lit(42)]),
                fn_call("isPopular", vec![var("p")]),
                fn_call("isFriendOf", vec![var("p"), var("p2")])
            ],
            tuple_lit(vec![var("p"), var("p2")])
        )
    );

    assert_parse_expr!(
        "? p <- Person
        ? p2 <- Person
        ! ((age p) == ((age p2) + 1))
        !> {p, p2}",
        query(
            vec![
                (vec!["p"], const_or_type_ref("Person"), false),
                (vec!["p2"], const_or_type_ref("Person"), false)
            ],
            vec![op_call(
                "==",
                fn_call("age", vec![var("p")]),
                op_call("+", fn_call("age", vec![var("p2")]), int_lit(1))
            ),],
            tuple_lit(vec![var("p"), var("p2")])
        )
    );

    assert_parse_expr!(
        "? p1 <- people
         ? p2 <- people
         ? f <- friends p1
         ? f <- friends p2
         !> f",
        query(
            vec![
                (vec!["p1"], var("people"), false),
                (vec!["p2"], var("people"), false),
                (vec!["f"], fn_call("friends", vec![var("p1")]), false),
                (vec!["f"], fn_call("friends", vec![var("p2")]), false),
            ],
            vec![],
            var("f"),
        )
    );

    assert_parse_expr!(
        "? p1, p2 <- people
         ? f <- friends p1
         ? f <- friends p2
         !> f",
        query(
            vec![
                (vec!["p1", "p2"], var("people"), false),
                (vec!["f"], fn_call("friends", vec![var("p1")]), false),
                (vec!["f"], fn_call("friends", vec![var("p2")]), false),
            ],
            vec![],
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
            vec![
                (vec!["a", "b", "c"], const_or_type_ref("Data"), false),
                (
                    vec!["diff"],
                    fn_call("dataDiff", vec![var("a"), var("b"), var("c")]),
                    false
                )
            ],
            vec![op_call(
                "<",
                int_lit(0),
                op_call("+", var("a"), op_call("+", var("b"), var("c")))
            )],
            tuple_lit(vec![
                var("diff"),
                tuple_lit(vec![var("a"), var("b"), var("c")])
            ]),
        )
    );

    assert_parse_expr!(
        "? bob <- Person
         ? joe <- Person
         ? bob <- job#Friend
         !> {bob, joe}",
        query(
            vec![
                (vec!["bob"], const_or_type_ref("Person"), false),
                (vec!["joe"], const_or_type_ref("Person"), false),
                (vec!["bob"], edge_prop(var("job"), "Friend"), false)
            ],
            vec![],
            tuple_lit(vec![var("bob"), var("joe")])
        )
    );

    assert_parse_expr!(
        "? p <- Person
         ? p <!- oldPeople
         !> p",
        query(
            vec![
                (vec!["p"], const_or_type_ref("Person"), false),
                (vec!["p"], var("oldPeople"), true)
            ],
            vec![],
            var("p")
        )
    );

    assert_parse_expr!(
        "? p <- Person
         ? p <- hello 1 2
         ? p <!- (a | b)#Friend
         !> inspect p",
        query(
            vec![
                (vec!["p"], const_or_type_ref("Person"), false),
                (
                    vec!["p"],
                    fn_call("hello", vec![int_lit(1), int_lit(2)]),
                    false
                ),
                (
                    vec!["p"],
                    edge_prop(op_call("|", var("a"), var("b")), "Friend"),
                    true
                )
            ],
            vec![],
            fn_call("inspect", vec![var("p")])
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
            vec![
                (vec!["p"], const_or_type_ref("Person"), false),
                (
                    vec!["p"],
                    fn_call("hello", vec![int_lit(1), int_lit(2)]),
                    false
                ),
                (
                    vec!["p"],
                    edge_prop(
                        query(
                            vec![(vec!["p2"], op_call("|", var("a"), var("b")), false)],
                            vec![],
                            var("p2")
                        ),
                        "Friend"
                    ),
                    true
                ),
                (
                    vec!["p"],
                    fn_call("friendsWithoutFriends", vec![var("p")]),
                    true
                )
            ],
            vec![],
            fn_call("inspect", vec![var("p")])
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
        tuple_lit(vec![prop_fn_ref("foo"), prop_fn_ref("bar")])
    );
}

#[test]
fn lambdas() {
    assert_parse_expr!(
        "x -> x + 1",
        lambda(vec!["x"], op_call("+", var("x"), int_lit(1)))
    );

    assert_parse_expr!(
        "x y -> x + y",
        lambda(vec!["x", "y"], op_call("+", var("x"), var("y")))
    );

    assert_parse_expr!(
        "x y -> inspect (z -> {x * (y * z), {x, y, z}}) x y",
        lambda(
            vec!["x", "y"],
            fn_call(
                "inspect",
                vec![
                    lambda(
                        vec!["z"],
                        tuple_lit(vec![
                            op_call("*", var("x"), op_call("*", var("y"), var("z"))),
                            tuple_lit(vec![var("x"), var("y"), var("z")])
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
            vec!["w", "x"],
            lambda(
                vec!["y"],
                lambda(
                    vec!["z"],
                    tuple_lit(vec![var("w"), var("x"), var("y"), var("z")])
                )
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
            vec![fn_call(
                "MyMod.doStuff",
                vec![fn_call("inspect", vec![int_lit(2)])]
            )]
        )
    );

    assert_parse_expr!(
        "2 |> inspect |> repeat 10",
        fn_call(
            "repeat",
            vec![fn_call("inspect", vec![int_lit(2)]), int_lit(10)]
        )
    );

    assert_parse_expr!(
        "[1, 2]
         |> joinWith {1,2,3}
         |> select (x -> isHappy x)
         |> sumBy (delta 1)",
        fn_call(
            "sumBy",
            vec![
                fn_call(
                    "select",
                    vec![
                        fn_call(
                            "joinWith",
                            vec![
                                list_lit(vec![int_lit(1), int_lit(2)]),
                                tuple_lit(vec![int_lit(1), int_lit(2), int_lit(3)])
                            ]
                        ),
                        lambda(vec!["x"], fn_call("isHappy", vec![var("x")]))
                    ]
                ),
                fn_call("delta", vec![int_lit(1)])
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
            vec!["x"],
            query(
                vec![(vec!["p"], const_or_type_ref("Person"), false)],
                vec![fn_call(
                    "thenDo",
                    vec![
                        fn_call(
                            "doStuff",
                            vec![
                                var("x"),
                                tuple_lit(vec![int_lit(2), int_lit(3), int_lit(4)])
                            ]
                        ),
                        lambda(
                            vec!["y"],
                            op_call(
                                "+",
                                var("x"),
                                fn_call(
                                    "join",
                                    vec![
                                        fn_call("toString", vec![var("y")],),
                                        tuple_lit(vec![int_lit(1), int_lit(2), int_lit(3)])
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
        quoted(fn_call("quoted", vec![var("expression")]))
    );

    assert_parse_expr!(
        "^(quoted (expression \"in quotes\" {1, 2, 3}))",
        quoted(fn_call(
            "quoted",
            vec![fn_call(
                "expression",
                vec![
                    string_lit("in quotes"),
                    tuple_lit(vec![int_lit(1), int_lit(2), int_lit(3)])
                ]
            )]
        ))
    );

    assert_parse_expr!(
        "^(quoted ~var in ~var2)",
        quoted(fn_call(
            "quoted",
            vec![unquoted(var("var")), var("in"), unquoted(var("var2"))]
        ))
    );

    assert_parse_expr!(
        "^(quoted ~(var {~var2, 123, ~var3, ^Cool}))",
        quoted(fn_call(
            "quoted",
            vec![unquoted(fn_call(
                "var",
                vec![tuple_lit(vec![
                    unquoted(var("var2")),
                    int_lit(123),
                    unquoted(var("var3")),
                    symbol("Cool")
                ])]
            ))]
        ))
    );

    assert_parse_expr!(
        "^(quoted ^Cool)",
        quoted(fn_call("quoted", vec![symbol("Cool")]))
    );

    assert_parse_expr!(
        "^(let f x = x + 1)",
        quoted_ast(fn_def("f", vec!["x"], op_call("+", var("x"), int_lit(1))))
    );

    assert_parse_expr!(
        "~(let f x = x + 1)",
        unquoted_ast(fn_def("f", vec!["x"], op_call("+", var("x"), int_lit(1))))
    );
}
