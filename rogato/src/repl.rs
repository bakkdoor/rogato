use std::fs::File;
use std::io::Read;
use std::path::Path;

use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::validate::MatchingBracketValidator;
use rustyline::{Cmd, Editor, EventHandler, KeyCode, KeyEvent, Modifiers};
use rustyline_derive::{Completer, Helper, Highlighter, Hinter, Validator};

use rogato_common::ast::ASTDepth;
use rogato_compiler::Codegen;
use rogato_interpreter::{EvalContext, EvalError, Evaluate};
use rogato_parser::{parse, parse_expr, ParseError, ParserContext};
use thiserror::Error;

#[derive(Completer, Helper, Highlighter, Hinter, Validator)]
struct InputValidator {
    #[rustyline(Validator)]
    brackets: MatchingBracketValidator,
}

fn validated_editor() -> Result<Editor<InputValidator>, ReadlineError> {
    let h = InputValidator {
        brackets: MatchingBracketValidator::new(),
    };
    let mut editor = Editor::new()?;
    editor.set_helper(Some(h));
    editor.bind_sequence(
        KeyEvent(KeyCode::Down, Modifiers::SHIFT),
        EventHandler::Simple(Cmd::Newline),
    );
    editor.bind_sequence(
        KeyEvent(KeyCode::Tab, Modifiers::NONE),
        EventHandler::Simple(Cmd::Insert(0, "  ".to_string())),
    );
    Ok(editor)
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn run_repl(files_to_load: &[String]) -> anyhow::Result<()> {
    println!("👾 rogātō ⌘ 🏷 ");
    print!("🖥  Interactive Shell ");
    println!("{VERSION} 🦀 \n");
    println!("Enter rogātō expressions below. You can add new lines via SHIFT-DOWN.\n");
    let mut eval_ctx = EvalContext::new();
    let parser_ctx = ParserContext::new();

    let mut counter = 0usize;

    let mut rl = validated_editor()?;
    rl.set_max_history_size(5000);

    let mut path_buf = dirs::home_dir().unwrap();
    path_buf.push(".rogato_history.txt");
    let history_file = path_buf.as_path();

    if rl.load_history(history_file).is_err() {
        println!("No previous history.");
    }

    for file_path_string in files_to_load {
        let file_path = Path::new(file_path_string.as_str());
        match File::open(file_path) {
            Ok(mut file) => {
                let mut buf = String::new();
                file.read_to_string(&mut buf).unwrap();
                match parse(buf.as_str(), &parser_ctx) {
                    Ok(program) => match program.evaluate(&mut eval_ctx) {
                        Ok(_) => {
                            println!("✅ {}", file_path.display());
                        }
                        Err(e) => {
                            eprintln!(
                                "❌ {file_path_string}\n\t\tFailed to evaluate file: {e}"
                            )
                        }
                    },
                    Err(e) => {
                        eprintln!("❌ {file_path_string}\n\t\tFailed to parse file: {e}")
                    }
                }
            }
            Err(error) => {
                eprintln!(
                    "❌ {file_path_string}\n\t\tCould not open source file: {error:?}"
                );
            }
        }
    }

    loop {
        let context = Codegen::new_context();
        let builder = context.create_builder();
        let module = context.create_module("rogato.repl");
        let fpm = Codegen::default_function_pass_manager(&module);
        let ee = Codegen::default_execution_engine(&module);
        let mut compiler = Codegen::new(&context, &module, &builder, &fpm, &ee);

        counter += 1;
        let readline = rl.readline(format!("{counter:03} >  ").as_str());
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                match parse_eval_print(&parser_ctx, &mut eval_ctx, &mut compiler, counter, &line) {
                    Ok(_) => {
                        continue;
                    }
                    Err(error) => {
                        eprintln!("REPL: {error}");
                        continue;
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("^D");
                break;
            }
            Err(err) => {
                eprintln!("Error: {err:?}");
                break;
            }
        }
    }

    rl.save_history(history_file)?;
    Ok(())
}

#[derive(Error, Debug)]
pub enum REPLError {
    #[error("EvalError: {0}")]
    Eval(EvalError),

    #[error("ParseError: {0}")]
    Parse(rogato_parser::ParseError),

    #[error("ReadlineError: {0}")]
    Readline(ReadlineError),
}

impl From<EvalError> for REPLError {
    fn from(e: EvalError) -> Self {
        REPLError::Eval(e)
    }
}

impl From<ParseError> for REPLError {
    fn from(e: ParseError) -> Self {
        REPLError::Parse(e)
    }
}

impl From<ReadlineError> for REPLError {
    fn from(e: ReadlineError) -> Self {
        REPLError::Readline(e)
    }
}

type F32JITFunc = unsafe extern "C" fn() -> f32;

fn parse_eval_print(
    parse_ctx: &ParserContext,
    eval_ctx: &mut EvalContext,
    compiler: &mut Codegen,
    counter: usize,
    code: &str,
) -> anyhow::Result<()> {
    match parse(code, parse_ctx) {
        Ok(ast) => {
            if rogato_common::util::is_debug_enabled() {
                println!("{counter:03} 🌳 {ast:?}\n\n{ast}\n");
            }

            if rogato_common::util::is_compilation_enabled() {
                compiler.codegen_program(&ast)?;
            }

            if compiler.module.get_function("main").is_some() {
                unsafe {
                    let main_fn = compiler
                        .execution_engine
                        .get_function::<F32JITFunc>("main")?;

                    let result = main_fn.call();
                    println!("{counter:03} ✅ {result}\n");
                    return Ok(());
                }
            }

            match ast.evaluate(eval_ctx) {
                Ok(val) => {
                    if val.ast_depth() > 5 {
                        println!("{counter:03} ✅\n{val}\n");
                    } else {
                        println!("{counter:03} ✅ {val}\n");
                    }
                    Ok(())
                }
                Err(e) => {
                    eprintln!("{counter:03} ❌ {e}\n");
                    Ok(())
                }
            }
        }

        Err(_) => {
            if rogato_common::util::is_compilation_enabled() {
                let func_name = format!("repl_{counter}");
                let code = format!("let {} = {}", func_name, code.trim());
                return match parse(code.as_str(), parse_ctx) {
                    Ok(ast) => {
                        if rogato_common::util::is_debug_enabled() {
                            println!("{counter:03} 🌳 {ast:?}\n\n{ast}\n");
                        }

                        compiler.codegen_program(&ast)?;

                        unsafe {
                            let tmp_function = compiler
                                .execution_engine
                                .get_function::<F32JITFunc>(func_name.as_str())?;

                            let result = tmp_function.call();
                            println!("{counter:03} ✅ {result}\n");
                            return Ok(());
                        }
                    }
                    Err(e) => {
                        eprintln!("{counter:03} ❌ {e:?}\n");
                        Ok(())
                    }
                };
            }

            match parse_expr(code.trim(), parse_ctx) {
                Ok(ast) => {
                    if rogato_common::util::is_debug_enabled() {
                        println!("{counter:03} 🌳 {ast:?}\n\n{ast}\n");
                    }

                    match ast.evaluate(eval_ctx) {
                        Ok(val) => {
                            if val.ast_depth() > 5 {
                                println!("{counter:03} ✅\n{val}\n");
                            } else {
                                println!("{counter:03} ✅ {val}\n");
                            }
                            Ok(())
                        }
                        Err(e) => {
                            eprintln!("{counter:03} ❌ {e}\n");
                            Ok(())
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{counter:03} ❌ {e:?}\n");
                    Ok(())
                }
            }
        }
    }
}
