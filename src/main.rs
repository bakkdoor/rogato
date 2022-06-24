mod fmodl;
use fmodl::ast::expression::{Expression, LiteralExpr};
use fmodl::grammar::parse;

fn main() {
    assert_eq!(
        parse("1+1"),
        Ok(Expression::Sum(
            Box::new(Expression::LiteralExpr(LiteralExpr::Int64(1))),
            Box::new(Expression::LiteralExpr(LiteralExpr::Int64(1)))
        ))
    );
    assert_eq!(
        parse("5*5"),
        Ok(Expression::Product(
            Box::new(Expression::LiteralExpr(LiteralExpr::Int64(5))),
            Box::new(Expression::LiteralExpr(LiteralExpr::Int64(5)))
        ))
    );
    assert_eq!(
        parse("2+3*4"),
        Ok(Expression::Sum(
            Box::new(Expression::LiteralExpr(LiteralExpr::Int64(2))),
            Box::new(Expression::Product(
                Box::new(Expression::LiteralExpr(LiteralExpr::Int64(3))),
                Box::new(Expression::LiteralExpr(LiteralExpr::Int64(4)))
            )),
        ))
    );
    assert_eq!(
        parse("(2+3) * 4"),
        Ok(Expression::Product(
            Box::new(Expression::Sum(
                Box::new(Expression::LiteralExpr(LiteralExpr::Int64(2))),
                Box::new(Expression::LiteralExpr(LiteralExpr::Int64(3))),
            )),
            Box::new(Expression::LiteralExpr(LiteralExpr::Int64(4)))
        ))
    );
    assert!(parse("(22+)+1").is_err());
    assert!(parse("1++1").is_err());
    assert!(parse("3)+1").is_err());

    for code in [
        "1",
        "1+1",
        "2+(3+4)",
        "1*2",
        "(2*3)*(3+5)",
        "2 * 3 + 4 * 8",
        "(+ 3 (* 4 5))",
        "(myFunction 2 3)",
        "(foo (bar 1) 2 3)",
        "(foo (bar (baz (* 3 (+ 4 5) (+ 6 7)))))",
        "(to-upper a b)",
        "(to-upper (__do-something-with__ a-var b_var))",
    ] {
        match parse(code) {
            Ok(expr) => println!("\n✅\t{}\n\n🧾\t{}\n\n\n", code, expr),
            Err(error) => println!("\n❌\t{}\n\n🧾\t{}\n\n\n", code, error),
        }
    }
}
