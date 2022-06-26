mod fmodl;
use std::fmt::Display;
use std::io;
use std::path::Path;

use fmodl::db;
use fmodl::parser::{parse, parse_expr};
use serde_json::Value;

const DB_PATH: &str = "./fmodl.db";

fn main() {
    let mut args = std::env::args();
    if let Some(_) = args.find(|a| a.eq("test")) {
        try_parse_root_defs();
        try_parse_expressions();
        do_db_stuff();
        return;
    }

    run_repl();
}

fn try_parse_root_defs() {
    for root_def in [
        "module MyModule",
        "module MyModule ( foo, bar, baz )",
        "module MyModule (  foo,     bar,   baz   )",
        "let squared x = x",
        "let add a b = 1 + b",
        "let addTwice a b = 2 * (a + b)",
        "let complexMath a b c =
            let x = 2 * (a + b * c),
                y = y * a + b * c,
                z = addTwice (squared (x * y)) (x * y)
            in
                y * z
        ",
    ] {
        print_parse_result(root_def, parse(root_def))
    }
}

fn try_parse_expressions() {
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
        println!("Found edges: {}", edges.len());
        for edge in edges.iter() {
            println!("Got edge: {:?}", edge)
        }
        edges
    })
    .map_err(print_error);

    println!("DB Query Result: {:?}", result);
}

fn print_parse_result<T: Display, E: Display>(code: &str, result: Result<T, E>) {
    match result {
        Ok(expr) => println!("\n✅\t{}\n\n🧾\t{}\n\n", code, expr),
        Err(error) => println!("\n❌\t{}\n\n🧾\t{}\n\n", code, error),
    }
}

fn print_error<E: std::fmt::Debug>(error: E) -> E {
    eprintln!("Error doing DB stuff: {:?}", error);
    error
}

fn run_repl() {
    loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        match parse(buffer.as_str()) {
            Ok(exp) => {
                println!("OK> {:?}", exp);
            }
            Err(err) => {
                eprintln!("Error> {:?}", err)
            }
        }
    }
}
