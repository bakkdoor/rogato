#[cfg(test)]
mod parser_tests {
    use crate::fmodl::ast::expression::{Expression::*, FnDefArgs, Literal::*};
    use crate::fmodl::ast::AST::FnDef;
    use crate::fmodl::parser::{parse, parse_expr};

    fn s<S: ToString>(s: S) -> String {
        s.to_string()
    }

    fn b<T>(t: T) -> Box<T> {
        Box::new(t)
    }

    #[test]
    fn fn_defs() {
        assert_eq!(
            parse("let id x = x"),
            Ok(FnDef(s("id"), FnDefArgs::new(vec![s("x")]), b(Var(s("x")))))
        );

        assert_eq!(
            parse("let add a b = a + b"),
            Ok(FnDef(
                s("add"),
                FnDefArgs::new(vec![s("a"), s("b")]),
                b(Sum(b(Var(s("a"))), b(Var(s("b")))))
            ))
        );
    }

    #[test]
    fn expressions() {
        assert_eq!(
            parse_expr("1+1"),
            Ok(Sum(b(Literal(Int64Lit(1))), b(Literal(Int64Lit(1)))))
        );

        assert_eq!(
            parse_expr("5*5"),
            Ok(Product(b(Literal(Int64Lit(5))), b(Literal(Int64Lit(5)))))
        );

        assert_eq!(
            parse_expr("2+3*4"),
            Ok(Sum(
                b(Literal(Int64Lit(2))),
                b(Product(b(Literal(Int64Lit(3))), b(Literal(Int64Lit(4))))),
            ))
        );

        assert_eq!(
            parse_expr("(2+3) * 4"),
            Ok(Product(
                b(Sum(b(Literal(Int64Lit(2))), b(Literal(Int64Lit(3))),)),
                b(Literal(Int64Lit(4)))
            ))
        );

        assert!(parse_expr("(22+)+1").is_err());
        assert!(parse_expr("1++1").is_err());
        assert!(parse_expr("3)+1").is_err());
    }
}
