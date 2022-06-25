#[cfg(test)]
mod parser_tests {
    use crate::fmodl::ast::expression::LetBindings;
    use crate::fmodl::ast::AST::FnDef;
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

        assert!(parse_expr("(22+)+1").is_err());
        assert!(parse_expr("1++1").is_err());
        assert!(parse_expr("3)+1").is_err());
    }
}
