#[cfg(test)]
mod parser_tests {
    use crate::fmodl::ast::expression::{FnCallArgs, LetBindings};
    use crate::fmodl::ast::module_def::ModuleExports;
    use crate::fmodl::ast::AST::{FnDef, ModuleDef};
    use crate::fmodl::ast::{
        expression::{
            Expression::{self, *},
            FnDefArgs,
            Literal::{self, *},
        },
        AST,
    };
    use crate::fmodl::parser::{parse, parse_expr};

    fn lit(lit: Literal) -> Box<Expression> {
        Box::new(Lit(lit))
    }

    fn var(id: &str) -> Box<Expression> {
        Box::new(Var(id.to_string()))
    }

    fn sum(a: Box<Expression>, b: Box<Expression>) -> Box<Expression> {
        Box::new(Sum(a, b))
    }

    fn product(a: Box<Expression>, b: Box<Expression>) -> Box<Expression> {
        Box::new(Product(a, b))
    }

    fn fn_def(id: &str, args: Vec<&str>, body: Box<Expression>) -> AST {
        FnDef(id.to_string(), fn_def_args(args), body)
    }

    fn fn_def_args(args: Vec<&str>) -> FnDefArgs {
        FnDefArgs::new(Vec::from_iter(args.iter().map(|a| a.to_string())))
    }

    fn let_exp(bindings: Vec<(&str, Box<Expression>)>, body: Box<Expression>) -> Box<Expression> {
        let bindings: Vec<(String, Expression)> = bindings
            .iter()
            .cloned()
            .map(|(name, expr)| (name.to_string(), *expr))
            .collect();

        Box::new(Let(LetBindings::new(bindings), body))
    }

    fn module_def(id: &str, exports: Vec<&str>) -> AST {
        ModuleDef(id.to_string(), module_def_exports(exports))
    }

    fn module_def_exports(exports: Vec<&str>) -> ModuleExports {
        ModuleExports::new(Vec::from_iter(exports.iter().map(|e| e.to_string())))
    }

    fn call_args(args: Vec<Box<Expression>>) -> FnCallArgs {
        let mut args_unboxed = Vec::new();
        for a in args {
            args_unboxed.push(*a)
        }
        FnCallArgs::new(args_unboxed)
    }

    fn fn_call(id: &str, args: Vec<Box<Expression>>) -> Box<Expression> {
        Box::new(Expression::FnCall(
            id.to_string(),
            Box::new(call_args(args)),
        ))
    }

    fn op_call(id: &str, args: Vec<Box<Expression>>) -> Box<Expression> {
        Box::new(Expression::OpCall(
            id.to_string(),
            Box::new(call_args(args)),
        ))
    }

    #[test]
    fn fn_defs() {
        assert_eq!(parse("let id x = x"), Ok(fn_def("id", vec!["x"], var("x"))));

        assert_eq!(
            parse("let add a b = a + b"),
            Ok(fn_def("add", vec!["a", "b"], sum(var("a"), var("b"))))
        );

        assert_eq!(
            parse("let add a b c = a + b * (c * a)"),
            Ok(fn_def(
                "add",
                vec!["a", "b", "c"],
                sum(var("a"), product(var("b"), product(var("c"), var("a"))))
            ))
        );

        assert_eq!(
            parse("let add1 a = 1 + a"),
            Ok(fn_def("add1", vec!["a"], sum(lit(Int64Lit(1)), var("a"))))
        );

        assert_eq!(
            parse("\nlet add1and2 = 1 + 2\n"),
            Ok(fn_def(
                "add1and2",
                vec![],
                sum(lit(Int64Lit(1)), lit(Int64Lit(2)))
            ))
        );

        assert_eq!(
            parse("let foo a b = bar a (baz 1)"),
            Ok(fn_def(
                "foo",
                vec!["a", "b"],
                fn_call(
                    "bar",
                    vec![var("a"), fn_call("baz", vec![lit(Int64Lit(1))])]
                )
            ))
        );
    }

    #[test]
    fn module_defs() {
        assert_eq!(parse("module MyModule"), Ok(module_def("MyModule", vec![])));
        assert_eq!(
            parse("module MyModule ()"),
            Ok(module_def("MyModule", vec![]))
        );
        assert_eq!(
            parse("module MyModule (    )"),
            Ok(module_def("MyModule", vec![]))
        );
        assert_eq!(
            parse("module MyModule (\n\n)"),
            Ok(module_def("MyModule", vec![]))
        );
        assert_eq!(
            parse("module MyModule (Foo_bar-baz)"),
            Ok(module_def("MyModule", vec!["Foo_bar-baz"]))
        );
        assert_eq!(
            parse("module MyModule (foo, bar)"),
            Ok(module_def("MyModule", vec!["foo", "bar"]))
        );
        assert_eq!(
            parse("module MyModule ( func1, func2, func3 )"),
            Ok(module_def("MyModule", vec!["func1", "func2", "func3"]))
        );
    }

    #[test]
    fn expressions() {
        assert_eq!(parse_expr("1"), Ok(lit(Int64Lit(1))));
        assert_eq!(
            parse_expr("\"Hello, world!\""),
            Ok(lit(StringLit("Hello, world!".to_string())))
        );

        assert_eq!(
            parse_expr("1+1"),
            Ok(sum(lit(Int64Lit(1)), lit(Int64Lit(1))))
        );

        assert_eq!(
            parse_expr("5*5"),
            Ok(product(lit(Int64Lit(5)), lit(Int64Lit(5))))
        );

        assert_eq!(
            parse_expr("2+3*4"),
            Ok(sum(
                lit(Int64Lit(2)),
                product(lit(Int64Lit(3)), lit(Int64Lit(4)))
            ))
        );

        assert_eq!(
            parse_expr("(2+3) * 4"),
            Ok(product(
                sum(lit(Int64Lit(2)), lit(Int64Lit(3))),
                lit(Int64Lit(4))
            ))
        );

        assert_eq!(
            parse_expr("let x = 1, y = 2 in x + y"),
            Ok(let_exp(
                vec![("x", lit(Int64Lit(1))), ("y", lit(Int64Lit(2)))],
                sum(var("x"), var("y"))
            ))
        );

        assert_eq!(
            parse_expr("add 1 2"),
            Ok(fn_call("add", vec![lit(Int64Lit(1)), lit(Int64Lit(2))]))
        );

        assert_eq!(
            parse_expr("1 != 2"),
            Ok(op_call("!=", vec![lit(Int64Lit(1)), lit(Int64Lit(2))]))
        );

        assert!(parse_expr("(22+)+1").is_err());
        assert!(parse_expr("1++1").is_err());
        assert!(parse_expr("3)+1").is_err());
    }
}
