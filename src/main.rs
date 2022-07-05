mod fmodl;
use std::fmt::Display;
use std::io::{self, Read};
use std::path::Path;

use fmodl::db;
use fmodl::parser::{parse, parse_expr};
use indent_write::indentable::Indentable;
use serde_json::Value;
use std::fs::File;

use crate::fmodl::util::print_error;

const DB_PATH: &str = "./fmodl.db";

#[cfg(test)]
mod tests;

fn main() {
    let args = std::env::args().skip(1);
    if args.len() == 0 {
        println!("No arguments given, but required.");
        print_help();
        return;
    }
    let mut help_required = false;
    for arg in args {
        match arg.as_str() {
            "help" => help_required = true,
            "repl" => {
                println!("Running REPL");
                run_repl();
            }
            "parse" => {
                println!("Running parse tests");
                try_parse_root_defs();
                try_parse_expressions();
            }
            "examples" => {
                println!("Trying to parse example files");
                match std::fs::read_dir(Path::new("examples/")) {
                    Ok(rd) => {
                        for e in rd {
                            let dir_entry = e.unwrap();
                            match File::open(dir_entry.path()) {
                                Ok(mut file) => {
                                    let mut buf = String::new();
                                    file.read_to_string(&mut buf).unwrap();
                                    println!("\n📂\t{}", dir_entry.path().display());
                                    print_parse_result(buf.as_str(), parse(buf.as_str()))
                                }
                                Err(error) => {
                                    println!("Could not open example source file: {:?}", error)
                                }
                            }
                        }
                    }
                    Err(_) => {}
                }
            }
            "db" => {
                println!("Running db tests");
                do_db_stuff();
            }
            _ => {
                println!("Unknown argument: {:?}", arg);
                help_required = true;
            }
        }
    }

    if help_required {
        print_help()
    }
}

fn print_help() {
    println!("Possible arguments:");
    println!("  help\n  repl\n  parse\n  examples\n  db");
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

    let index_result = db::index_prop(&datastore, "name");
    println!("index result for 'name': {:?}", index_result);

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

    db::do_stuff(&datastore);
}

fn print_parse_result<T: Display, E: Display>(code: &str, result: Result<T, E>) {
    let lines = code.split("\n");
    let line_count = Vec::from_iter(lines.to_owned()).len();
    let (_, code_with_line_numbers) = lines.fold((1, String::new()), |(counter, acc), line| {
        let mut string = format!("{}\n{:02}  {}", acc, counter, line);
        if line_count > 100 {
            string = format!("{}\n{:03}  {}", acc, counter, line)
        }
        if line_count > 1000 {
            string = format!("{}\n{:03}  {}", acc, counter, line)
        }

        (counter + 1, string)
    });

    match result {
        Ok(expr) => println!(
            "✅\t{}\n\n🧾 ✅\n{}\n\n",
            code_with_line_numbers,
            expr.indented("\t")
        ),
        Err(error) => println!("\n❌\t{}\n\n❌\t{}\n\n", code_with_line_numbers, error),
    }
}

fn run_repl() {
    loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        match parse(buffer.as_str()) {
            Ok(exp) => {
                println!("OK> {:?}\n{}", exp, exp);
            }
            Err(err) => {
                eprintln!("Error> {:?}", err)
            }
        }
    }
}
