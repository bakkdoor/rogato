use crate::tests::rogato::{
    commented, int_type, list_lit, root_comment, string_type, struct_lit, struct_type, tuple_type,
    type_def, type_ref,
};
#[cfg(test)]
use crate::{assert_parse, assert_parse_ast, assert_parse_expr};

use super::{const_or_type_ref, query};
#[cfg(test)]
use super::{
    fn_call, fn_def, int_lit, let_expr, module_def, op_call, parse_expr, product, program,
    string_lit, sum, tuple_lit, var,
};

#[test]
fn fn_defs() {
    assert_parse_ast!("let id x = x", fn_def("id", vec!["x"], var("x")));

    assert_parse_ast!(
        "let add a b = a + b",
        fn_def("add", vec!["a", "b"], sum(var("a"), var("b")))
    );

    assert_parse_ast!(
        "let add a b c = a + b * (c * a)",
        fn_def(
            "add",
            vec!["a", "b", "c"],
            sum(var("a"), product(var("b"), product(var("c"), var("a"))))
        )
    );

    assert_parse_ast!(
        "let add1 a = 1 + a",
        fn_def("add1", vec!["a"], sum(int_lit(1), var("a")))
    );

    assert_parse_ast!(
        "\nlet add1and2 = 1 + 2\n",
        fn_def("add1and2", vec![], sum(int_lit(1), int_lit(2)))
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
            let x = a + b,
                y = x * a,
                z = y * b
            in
                {x, y, z, {x, y}}
        ",
        fn_def(
            "foo",
            vec!["a", "b"],
            let_expr(
                vec![
                    ("x", sum(var("a"), var("b"))),
                    ("y", product(var("x"), var("a"))),
                    ("z", product(var("y"), var("b"))),
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
    assert_parse_ast!("module MyModule ()", module_def("MyModule", vec![]));
    assert_parse_ast!("module MyModule (    )", module_def("MyModule", vec![]));
    assert_parse_ast!("module MyModule (\n\n)", module_def("MyModule", vec![]));
    assert_parse_ast!(
        "module MyModule (Foo_bar-baz)",
        module_def("MyModule", vec!["Foo_bar-baz"])
    );
    assert_parse_ast!(
        "module MyModule (foo, bar)",
        module_def("MyModule", vec!["foo", "bar"])
    );
    assert_parse_ast!(
        "module MyModule ( func1, func2, func3 )",
        module_def("MyModule", vec!["func1", "func2", "func3"])
    );
}

#[test]
fn arithmetic_expressions() {
    assert_parse_expr!("1+1", sum(int_lit(1), int_lit(1)));

    assert_parse_expr!("5*5", product(int_lit(5), int_lit(5)));

    assert_parse_expr!("2+3*4", sum(int_lit(2), product(int_lit(3), int_lit(4)),));

    assert_parse_expr!(
        "(2+3) * 4",
        product(sum(int_lit(2), int_lit(3)), int_lit(4))
    );

    assert_parse_expr!(
        "let x = 1, y = 2 in x + y",
        let_expr(
            vec![("x", int_lit(1)), ("y", int_lit(2))],
            sum(var("x"), var("y")),
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
        tuple_lit(vec![int_lit(1), sum(int_lit(2), int_lit(3)), int_lit(4)])
    );

    assert_parse_expr!(
        "{ 1, 2 + 3, 4 * 5 }",
        tuple_lit(vec![
            int_lit(1),
            sum(int_lit(2), int_lit(3)),
            product(int_lit(4), int_lit(5))
        ])
    );

    assert_parse_expr!(
        "{ 1, a + b, c * d }",
        tuple_lit(vec![
            int_lit(1),
            sum(var("a"), var("b")),
            product(var("c"), var("d"))
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
        op_call("<=", int_lit(1), sum(int_lit(2), int_lit(3)))
    );

    assert_parse_expr!(
        "(2 + 3) <= foo",
        op_call("<=", sum(int_lit(2), int_lit(3)), var("foo"))
    );

    assert_parse_expr!(
        "(2 + 3) <= (foo <!> (bar <=> baz))",
        op_call(
            "<=",
            sum(int_lit(2), int_lit(3)),
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
            let_expr(vec![("x", int_lit(1))], product(var("x"), int_lit(2)))
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
        "let name = \"John Connor\",
             age = 12,
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
            friendsOfFriends = [],
            friendsNames =
                (List.map friendsOfFriends name)
         in
            {friends, List.count friends}",
        let_expr(
            vec![
                ("friendsOfFriends", list_lit(vec![])),
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
        ! (isOlderThan p 42)
        ! (isPopular p)
        !> p",
        query(
            vec![(vec!["p"], const_or_type_ref("Person"))],
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
        ! (isOlderThan p 42)
        ! (isPopular p)
        ! (isFriendOf p p2)
        !> {p, p2}",
        query(
            vec![
                (vec!["p"], const_or_type_ref("Person")),
                (vec!["p2"], const_or_type_ref("Person"))
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
                (vec!["p"], const_or_type_ref("Person")),
                (vec!["p2"], const_or_type_ref("Person"))
            ],
            vec![op_call(
                "==",
                fn_call("age", vec![var("p")]),
                sum(fn_call("age", vec![var("p2")]), int_lit(1))
            ),],
            tuple_lit(vec![var("p"), var("p2")])
        )
    );

    assert_parse_expr!(
        "? p1 <- people
         ? p2 <- people
         ? f <- (friends p1)
         ? f <- (friends p2)
         !> f",
        query(
            vec![
                (vec!["p1"], var("people")),
                (vec!["p2"], var("people")),
                (vec!["f"], fn_call("friends", vec![var("p1")])),
                (vec!["f"], fn_call("friends", vec![var("p2")])),
            ],
            vec![],
            var("f"),
        )
    );

    assert_parse_expr!(
        "? p1, p2 <- people
         ? f <- (friends p1)
         ? f <- (friends p2)
         !> f",
        query(
            vec![
                (vec!["p1", "p2"], var("people")),
                (vec!["f"], fn_call("friends", vec![var("p1")])),
                (vec!["f"], fn_call("friends", vec![var("p2")])),
            ],
            vec![],
            var("f"),
        )
    );

    assert_parse_expr!(
        "? a,
           b
           ,c <- Data
         ? diff <- (dataDiff a b c)
         ! (0 < (a + (b + c)))
         !> {diff, {a, b, c}}",
        query(
            vec![
                (vec!["a", "b", "c"], const_or_type_ref("Data")),
                (
                    vec!["diff"],
                    fn_call("dataDiff", vec![var("a"), var("b"), var("c")])
                )
            ],
            vec![op_call(
                "<",
                int_lit(0),
                sum(var("a"), sum(var("b"), var("c")))
            )],
            tuple_lit(vec![
                var("diff"),
                tuple_lit(vec![var("a"), var("b"), var("c")])
            ]),
        )
    )
}
