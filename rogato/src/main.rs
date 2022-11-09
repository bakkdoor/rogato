#[allow(unused_imports)]
use rogato_interpreter::{EvalContext, Evaluate};
use rogato_parser::{parse, ParserContext};

use clap::Parser;
use indent_write::indentable::Indentable;
use std::fmt::{Debug, Display};
use std::fs::File;
use std::io::Read;
use std::path::Path;

mod repl;

// const DB_PATH: &str = "./rogato.db";

#[derive(Parser, Debug)]
#[clap(author,version,about,long_about=None)]
struct CLIArgs {
    #[clap(short, long, value_parser)]
    name: String,
}

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args();
    if args.len() == 1 {
        println!("No arguments given, but required.");
        print_help();
        return Ok(());
    }
    let mut help_required = false;
    let parser_ctx = ParserContext::new();

    match args.nth(1).unwrap().as_str() {
        "help" => help_required = true,
        "repl" => {
            repl::run_repl(args)?;
        }
        "compile" => todo!(),
        file => {
            println!("Attempting file parse: {}", file);
            let file_path = Path::new(file);
            if file_path.exists() {
                read_parse_file(file_path, &parser_ctx);
            } else {
                eprintln!("File not found: {:?}. Aborting.", file);
                help_required = true;
            }
        }
    }

    if help_required {
        print_help()
    }

    Ok(())
}

fn read_parse_file(file_path: &Path, parser_ctx: &ParserContext) {
    match File::open(file_path) {
        Ok(mut file) => {
            let mut buf = String::new();
            file.read_to_string(&mut buf).unwrap();
            println!("\n📂\t{}", file_path.display());
            let parse_result = parse(buf.as_str(), parser_ctx);
            print_parse_result(buf.as_str(), &parse_result);
        }
        Err(error) => {
            println!("Could not open source file: {:?}", error);
        }
    }
}

fn print_help() {
    println!("Possible arguments:");
    println!("  help\n  repl\n  db\n  <source file path>");
}

fn print_parse_result<T: Display, E: Display>(code: &str, result: &Result<T, E>) {
    let lines = code.split('\n');
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
        Ok(expr) => println!("🌳 ✅\n{}\n\n", expr.indented("\t")),
        Err(error) => println!("❌{}\n\n❌\t{}\n\n", code_with_line_numbers, error),
    }
}
