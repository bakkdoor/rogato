mod fmodl;
use std::fmt::Display;
use std::path::Path;

use fmodl::db;
use fmodl::parser::{parse, parse_expr};
use serde_json::Value;

const DB_PATH: &str = "./fmodl.db";

fn main() {
    // assert_eq!(
    //     parse("1+1"),
    //     Ok(Sum(
    //         Box::new(Literal(Int64Lit(1))),
    //         Box::new(Literal(Int64Lit(1)))
    //     ))
    // );
    // assert_eq!(
    //     parse("5*5"),
    //     Ok(Product(
    //         Box::new(Literal(Int64Lit(5))),
    //         Box::new(Literal(Int64Lit(5)))
    //     ))
    // );
    // assert_eq!(
    //     parse("2+3*4"),
    //     Ok(Sum(
    //         Box::new(Literal(Int64Lit(2))),
    //         Box::new(Product(
    //             Box::new(Literal(Int64Lit(3))),
    //             Box::new(Literal(Int64Lit(4)))
    //         )),
    //     ))
    // );
    // assert_eq!(
    //     parse("(2+3) * 4"),
    //     Ok(Product(
    //         Box::new(Sum(
    //             Box::new(Literal(Int64Lit(2))),
    //             Box::new(Literal(Int64Lit(3))),
    //         )),
    //         Box::new(Literal(Int64Lit(4)))
    //     ))
    // );
    // assert!(parse("(22+)+1").is_err());
    // assert!(parse("1++1").is_err());
    // assert!(parse("3)+1").is_err());

    for root_def in [
        "let squared x = (x * x)",
        "let add a b = a + b",
        "let addTwice a b = 2 * (a + b)",
    ] {
        print_parse_result(root_def, parse(root_def))
    }

    for expr_code in [
        "1",
        "1+1",
        "2+(3+4)",
        "1*2",
        "(2*3)*(3+5)",
        "2 * 3 + 4 * 8",
        "3 + (4 * 5)",
        "myFunction 2 3",
        "foo (bar 1)",
        "foo (bar 1) 2 3",
        "foo (bar (baz (3 * ((4 * 5) * (6 + 7)))))",
        "to-upper a b",
        "      to-upper   (  __do-something-with__         a-var        b_var      )   ",
    ] {
        print_parse_result(expr_code, parse_expr(expr_code))
    }

    do_db_stuff();
}

fn print_parse_result<T: Display, E: Display>(code: &str, result: Result<T, E>) {
    match result {
        Ok(expr) => println!("\n✅\t{}\n\n🧾\t{}\n\n", code, expr),
        Err(error) => println!("\n❌\t{}\n\n🧾\t{}\n\n", code, error),
    }
}

fn do_db_stuff() {
    println!("Opening DB @ {}", DB_PATH);

    let datastore = db::open(Path::new(DB_PATH)).map_err(print_error).unwrap();

    let result = db::query_prop(
        &datastore,
        "name",
        Value::String(String::from("John Connor")),
    )
    .map(|edges| {
        println!("Found edges: {}", edges.iter().count());
        for edge in edges.iter() {
            println!("Got edge: {:?}", edge)
        }
        edges
    })
    .map_err(print_error);

    println!("DB Query Result: {:?}", result);
}

fn print_error<E: std::fmt::Debug>(error: E) -> E {
    eprintln!("Error doing DB stuff: {:?}", error);
    error
}
