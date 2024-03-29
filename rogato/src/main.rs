#[allow(unused_imports)]
use rogato_interpreter::{EvalContext, Evaluate};
use rogato_parser::{parse, ParserContext};

use clap::Parser;
use indent_write::indentable::Indentable;
use std::collections::HashSet;
use std::fmt::Display;
use std::fs::File;
use std::io::Read;
use std::path::Path;

mod repl;

#[cfg(feature = "flame_it")]
extern crate flame;
#[cfg(feature = "flame_it")]
#[macro_use]
extern crate flamer;

// const DB_PATH: &str = "./rogato.db";

/// Doc comment
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct CLIArgs {
    #[command(subcommand)]
    command: Command,
}

/// Doc comment
#[derive(Parser, PartialEq, Eq, Debug)]
#[command(about = "Which rogātō subcommand to run")]
enum Command {
    #[command(name = "repl", about = "Runs the REPL")]
    RunRepl(ReplInfo),

    #[command(name = "eval", about = "Evaluate / Runs the given source file")]
    EvaluateFile(FileInfo),

    #[command(name = "compile", about = "Compiles the given source file")]
    CompileFile(FileInfo),
}

#[derive(Parser, PartialEq, Eq, Debug)]
struct FileInfo {
    #[arg(long, short)]
    files: Vec<String>,
}

#[derive(Parser, PartialEq, Eq, Debug)]
struct ReplInfo {
    // Files to parse & load before running REPL
    #[arg(alias = "load", long, short = 'l')]
    preload: Vec<String>,
}

fn std_lib_preloads() -> HashSet<String> {
    HashSet::from([
        "lib/Std.roga".into(),
        "lib/Std/List.roga".into(),
        "lib/Std/Map.roga".into(),
    ])
}

#[cfg_attr(feature = "flame_it", flame)]
fn main() -> anyhow::Result<()> {
    let args = CLIArgs::parse();
    let parser_ctx = ParserContext::new();

    match args.command {
        Command::RunRepl(repl_info) => {
            let mut preloads: HashSet<String> = std_lib_preloads();
            preloads.extend(repl_info.preload);

            let unique_preloads: Vec<String> = preloads.into_iter().collect();
            repl::run_repl(&unique_preloads)?;
        }
        Command::EvaluateFile(file_info) => {
            for file in file_info.files.iter() {
                println!("Attempting file parse: {file}");
                let file_path = Path::new(file);
                if file_path.exists() {
                    read_parse_file(file_path, &parser_ctx);
                } else {
                    eprintln!("File not found: {file:?}. Aborting.");
                }
            }
        }
        Command::CompileFile(_file_info) => todo!(),
    }

    #[cfg(feature = "flame_it")]
    flame::dump_html(File::create("flamegraph.html").unwrap()).unwrap();

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
            println!("Could not open source file: {error:?}");
        }
    }
}

fn print_parse_result<T: Display, E: Display>(code: &str, result: &Result<T, E>) {
    let lines = code.split('\n');
    let line_count = Vec::from_iter(lines.to_owned()).len();
    let (_, code_with_line_numbers) = lines.fold((1, String::new()), |(counter, acc), line| {
        let mut string = format!("{acc}\n{counter:02}  {line}");
        if line_count > 100 {
            string = format!("{acc}\n{counter:03}  {line}")
        }
        if line_count > 1000 {
            string = format!("{acc}\n{counter:03}  {line}")
        }

        (counter + 1, string)
    });

    match result {
        Ok(expr) => println!("🌳 ✅\n{}\n\n", expr.indented("\t")),
        Err(error) => println!("❌{code_with_line_numbers}\n\n❌\t{error}\n\n"),
    }
}
