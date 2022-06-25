#[cfg(test)]
mod parser_tests {
    use crate::fmodl::ast::expression::{Expression::*, FnDefArgs, Literal::*};
    use crate::fmodl::ast::AST::FnDef;
    use crate::fmodl::parser::{parse, parse_expr};

    fn s<S: ToString>(s: S) -> String {
        s.to_string()
    }

    #[test]
    fn fn_defs() {
        assert_eq!(
            parse("let id x = x"),
            Ok(FnDef(
                s("id"),
                FnDefArgs::new(vec![s("x")]),
                Box::new(Var(s("x")))
            ))
        );

        assert_eq!(
            parse("let add a b = a + b"),
            Ok(FnDef(
                s("add"),
                FnDefArgs::new(vec![s("a"), s("b")]),
                Box::new(Sum(Box::new(Var(s("a"))), Box::new(Var(s("b")))))
            ))
        );
    }

    #[test]
    fn expressions() {
        assert_eq!(
            parse_expr("1+1"),
            Ok(Sum(
                Box::new(Literal(Int64Lit(1))),
                Box::new(Literal(Int64Lit(1)))
            ))
        );

        assert_eq!(
            parse_expr("5*5"),
            Ok(Product(
                Box::new(Literal(Int64Lit(5))),
                Box::new(Literal(Int64Lit(5)))
            ))
        );

        assert_eq!(
            parse_expr("2+3*4"),
            Ok(Sum(
                Box::new(Literal(Int64Lit(2))),
                Box::new(Product(
                    Box::new(Literal(Int64Lit(3))),
                    Box::new(Literal(Int64Lit(4)))
                )),
            ))
        );

        assert_eq!(
            parse_expr("(2+3) * 4"),
            Ok(Product(
                Box::new(Sum(
                    Box::new(Literal(Int64Lit(2))),
                    Box::new(Literal(Int64Lit(3))),
                )),
                Box::new(Literal(Int64Lit(4)))
            ))
        );

        assert!(parse_expr("(22+)+1").is_err());
        assert!(parse_expr("1++1").is_err());
        assert!(parse_expr("3)+1").is_err());
    }
}
