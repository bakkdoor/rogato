use crate::tests::fmodl::{commented, root_comment, struct_lit};
#[cfg(test)]
use crate::{assert_parse, assert_parse_ast, assert_parse_expr};

#[cfg(test)]
use super::{
    fn_call, fn_def, int_lit, let_exp, module_def, op_call, parse_expr, product, program,
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
        let_exp(
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
            let_exp(vec![("x", int_lit(1))], product(var("x"), int_lit(2)))
        )
    );
}
