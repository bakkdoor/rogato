mod fmodl;
use std::path::Path;

use fmodl::ast::expression::{Expression::*, Literal::*};
use fmodl::db;
use fmodl::grammar::parse;
use serde_json::Value;

fn main() {
    assert_eq!(
        parse("1+1"),
        Ok(Sum(
            Box::new(Literal(Int64Lit(1))),
            Box::new(Literal(Int64Lit(1)))
        ))
    );
    assert_eq!(
        parse("5*5"),
        Ok(Product(
            Box::new(Literal(Int64Lit(5))),
            Box::new(Literal(Int64Lit(5)))
        ))
    );
    assert_eq!(
        parse("2+3*4"),
        Ok(Sum(
            Box::new(Literal(Int64Lit(2))),
            Box::new(Product(
                Box::new(Literal(Int64Lit(3))),
                Box::new(Literal(Int64Lit(4)))
            )),
        ))
    );
    assert_eq!(
        parse("(2+3) * 4"),
        Ok(Product(
            Box::new(Sum(
                Box::new(Literal(Int64Lit(2))),
                Box::new(Literal(Int64Lit(3))),
            )),
            Box::new(Literal(Int64Lit(4)))
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
        "(      to-upper   (  __do-something-with__         a-var        b_var      )   )",
    ] {
        match parse(code) {
            Ok(expr) => println!("\n✅\t{}\n\n🧾\t{}\n\n", code, expr),
            Err(error) => println!("\n❌\t{}\n\n🧾\t{}\n\n", code, error),
        }
    }

    match db::open(Path::new("./fmodl.db")) {
        Ok(db) => {
            let edges = db::query_prop(db, "name", Value::String(String::from("John Connor")));
            println!("Found edges: {}", edges.iter().count());
            for edge in edges {
                println!("Got edge: {:?}", edge)
            }
        }
        Err(error) => {
            println!("Error: {:?}", error)
        }
    }
}
