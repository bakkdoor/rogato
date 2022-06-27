#[cfg(test)]
use super::{
    assert_parse, assert_parse_expr, fn_call, fn_def, let_exp, lit, module_def, op_call,
    parse_expr, product, sum, var, Int64Lit, StringLit,
};

#[test]
fn fn_defs() {
    assert_parse("let id x = x", fn_def("id", vec!["x"], var("x")));

    assert_parse(
        "let add a b = a + b",
        fn_def("add", vec!["a", "b"], sum(var("a"), var("b"))),
    );

    assert_parse(
        "let add a b c = a + b * (c * a)",
        fn_def(
            "add",
            vec!["a", "b", "c"],
            sum(var("a"), product(var("b"), product(var("c"), var("a")))),
        ),
    );

    assert_parse(
        "let add1 a = 1 + a",
        fn_def("add1", vec!["a"], sum(lit(Int64Lit(1)), var("a"))),
    );

    assert_parse(
        "\nlet add1and2 = 1 + 2\n",
        fn_def("add1and2", vec![], sum(lit(Int64Lit(1)), lit(Int64Lit(2)))),
    );

    assert_parse(
        "let foo a b = bar a (baz 1)",
        fn_def(
            "foo",
            vec!["a", "b"],
            fn_call(
                "bar",
                vec![var("a"), fn_call("baz", vec![lit(Int64Lit(1))])],
            ),
        ),
    );
}

#[test]
fn module_defs() {
    assert_parse("module MyModule", module_def("MyModule", vec![]));
    assert_parse("module MyModule ()", module_def("MyModule", vec![]));
    assert_parse("module MyModule (    )", module_def("MyModule", vec![]));
    assert_parse("module MyModule (\n\n)", module_def("MyModule", vec![]));
    assert_parse(
        "module MyModule (Foo_bar-baz)",
        module_def("MyModule", vec!["Foo_bar-baz"]),
    );
    assert_parse(
        "module MyModule (foo, bar)",
        module_def("MyModule", vec!["foo", "bar"]),
    );
    assert_parse(
        "module MyModule ( func1, func2, func3 )",
        module_def("MyModule", vec!["func1", "func2", "func3"]),
    );
}

#[test]
fn arithmetic_expressions() {
    assert_parse_expr("1+1", sum(lit(Int64Lit(1)), lit(Int64Lit(1))));

    assert_parse_expr("5*5", product(lit(Int64Lit(5)), lit(Int64Lit(5))));

    assert_parse_expr(
        "2+3*4",
        sum(
            lit(Int64Lit(2)),
            product(lit(Int64Lit(3)), lit(Int64Lit(4))),
        ),
    );

    assert_parse_expr(
        "(2+3) * 4",
        product(sum(lit(Int64Lit(2)), lit(Int64Lit(3))), lit(Int64Lit(4))),
    );

    assert_parse_expr(
        "let x = 1, y = 2 in x + y",
        let_exp(
            vec![("x", lit(Int64Lit(1))), ("y", lit(Int64Lit(2)))],
            sum(var("x"), var("y")),
        ),
    );

    assert!(parse_expr("(22+)+1").is_err());
    assert!(parse_expr("1++1").is_err());
    assert!(parse_expr("3)+1").is_err());
}

#[test]
fn literals() {
    assert_parse_expr("1", lit(Int64Lit(1)));

    assert_parse_expr(
        "\"Hello, world!\"",
        lit(StringLit("Hello, world!".to_string())),
    );
}

#[test]
fn fn_calls() {
    assert_parse_expr(
        "add 1 2",
        fn_call("add", vec![lit(Int64Lit(1)), lit(Int64Lit(2))]),
    );
}

#[test]
fn op_calls() {
    assert_parse_expr(
        "1 != 2",
        op_call("!=", vec![lit(Int64Lit(1)), lit(Int64Lit(2))]),
    );
}
