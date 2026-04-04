#[allow(unused_imports)]
use rogato_compiler::Codegen;
use rogato_parser::{parse, ParserContext};

use clap::Parser;
use indent_write::indentable::Indentable;
use std::collections::HashSet;
use std::fmt::Display;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

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
    CompileFile(CompileOptions),
}

#[derive(Parser, PartialEq, Eq, Debug)]
struct FileInfo {
    #[arg(long, short)]
    files: Vec<String>,
}

#[derive(Parser, PartialEq, Eq, Debug)]
struct CompileOptions {
    #[arg(long, short = 'f')]
    files: Vec<String>,

    #[arg(long, short = 'o')]
    output: Option<PathBuf>,

    #[arg(long)]
    ir: bool,

    #[arg(long)]
    bc: bool,

    #[arg(long)]
    obj: bool,

    #[arg(long)]
    asm: bool,

    #[arg(long)]
    all: bool,
}

#[derive(Parser, PartialEq, Eq, Debug)]
struct ReplInfo {
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
        Command::CompileFile(compile_opts) => {
            let files = &compile_opts.files;
            if files.is_empty() {
                eprintln!("Error: No input files specified.");
                return Ok(());
            }

            for file in files {
                compile_file(file, &compile_opts)?;
            }
        }
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

fn compile_file(file_path: &str, opts: &CompileOptions) -> anyhow::Result<()> {
    let path = Path::new(file_path);
    if !path.exists() {
        eprintln!("Error: File not found: {file_path}");
        return Ok(());
    }

    let mut source_file = File::open(path)?;
    let mut source_code = String::new();
    source_file.read_to_string(&mut source_code)?;

    let ast = parse(source_code.as_str(), &ParserContext::new())
        .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    let context = Codegen::new_context();
    let builder = context.create_builder();
    let module_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("rogato_module");
    let module = context.create_module(module_name);
    let target_machine = Codegen::default_target_machine(&module);
    let execution_engine = Codegen::default_execution_engine(&module);

    let mut compiler = Codegen::new(
        &context,
        &module,
        &builder,
        &target_machine,
        &execution_engine,
    );

    compiler.init_stdlib();
    compiler.codegen_program(&ast)?;

    let base_path = opts
        .output
        .as_ref()
        .map(|p| p.join(path.file_stem().unwrap()))
        .unwrap_or_else(|| {
            path.parent()
                .map(|p| p.join(path.file_stem().unwrap()))
                .unwrap_or_else(|| PathBuf::from(path.file_stem().unwrap()))
        });

    let emit_all = opts.all;
    let any_ir = opts.ir || (!opts.bc && !opts.obj && !opts.asm);
    let any_bc = opts.bc || emit_all;
    let any_obj = opts.obj || emit_all;
    let any_asm = opts.asm || emit_all;

    if any_ir {
        let ir_path = base_path.with_extension("ll");
        compiler.write_ir_to_file(&ir_path)?;
        println!("Wrote LLVM IR: {}", ir_path.display());
    }

    if any_bc {
        let bc_path = base_path.with_extension("bc");
        compiler.write_bitcode_to_file(&bc_path)?;
        println!("Wrote LLVM bitcode: {}", bc_path.display());
    }

    if any_obj {
        let obj_path = base_path.with_extension("o");
        compiler.write_object_to_file(&obj_path)?;
        println!("Wrote object file: {}", obj_path.display());

        if let Err(e) = link_object_file(&obj_path, &base_path.with_extension("")) {
            eprintln!(
                "Warning: Linking failed: {}. Object file still available.",
                e
            );
        }
    }

    if any_asm {
        let asm_path = base_path.with_extension("s");
        compiler.write_assembly_to_file(&asm_path)?;
        println!("Wrote assembly: {}", asm_path.display());
    }

    Ok(())
}

fn link_object_file(obj_path: &Path, output_path: &Path) -> anyhow::Result<()> {
    let compiler = std::env::var("CC").unwrap_or_else(|_| "clang".to_string());
    let output = output_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("a.out");

    let result = ProcessCommand::new(&compiler)
        .arg(obj_path)
        .arg("-o")
        .arg(output)
        .output()?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(anyhow::anyhow!("Linker error: {}", stderr));
    }

    println!("Linked executable: ./{}", output);
    Ok(())
}
