use rustyline::error::ReadlineError;
use rustyline::validate::MatchingBracketValidator;
use rustyline::{Cmd, Editor, EventHandler, KeyCode, KeyEvent, Modifiers};
use rustyline_derive::{Completer, Helper, Highlighter, Hinter, Validator};

use rogato_common::ast::ASTDepth;
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

pub fn run_repl() -> anyhow::Result<()> {
    println!("👾 rogātō ⌘ 🏷 ");
    print!("🖥  Interactive Shell ");
    println!("{} 🦀 \n", VERSION);
    println!("Enter rogātō expressions below. You can add new lines via SHIFT-DOWN.\n");
    let mut eval_ctx = EvalContext::new();
    let parse_ctx = ParserContext::new();
    let mut counter = 0usize;

    // let mut rl = rustyline::Editor::<()>::new()?;
    let mut rl = validated_editor()?;

    let mut path_buf = dirs::home_dir().unwrap();
    path_buf.push(".rogato_history.txt");
    let history_file = path_buf.as_path();

    if rl.load_history(history_file).is_err() {
        println!("No previous history.");
    }

    loop {
        counter += 1;
        let readline = rl.readline(format!("{:03} >  ", counter).as_str());
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                parse_eval_print(&parse_ctx, &mut eval_ctx, counter, &line)?;
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
                eprintln!("Error: {:?}", err);
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

fn parse_eval_print(
    parse_ctx: &ParserContext,
    eval_ctx: &mut EvalContext,
    counter: usize,
    code: &str,
) -> anyhow::Result<()> {
    match parse(code, parse_ctx) {
        Ok(ast) => {
            if rogato_common::util::debug_enabled() {
                println!("{:03} 🌳 {:?}\n\n{}\n", counter, ast, ast);
            }
            match ast.evaluate(eval_ctx) {
                Ok(val) => {
                    if val.ast_depth() > 5 {
                        println!("{:03} ✅\n{}\n", counter, val);
                    } else {
                        println!("{:03} ✅ {}\n", counter, val);
                    }
                    Ok(())
                }
                Err(e) => {
                    eprintln!("{:03} ❌ {}\n", counter, e);
                    Ok(())
                }
            }
        }
        Err(_) => match parse_expr(code.trim(), parse_ctx) {
            Ok(ast) => {
                if rogato_common::util::debug_enabled() {
                    println!("{:03} 🌳 {:?}\n\n{}\n", counter, ast, ast);
                }
                match ast.evaluate(eval_ctx) {
                    Ok(val) => {
                        if val.ast_depth() > 5 {
                            println!("{:03} ✅\n{}\n", counter, val);
                        } else {
                            println!("{:03} ✅ {}\n", counter, val);
                        }
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("{:03} ❌ {}\n", counter, e);
                        Ok(())
                    }
                }
            }
            Err(e) => {
                eprintln!("{:03} ❌ {:?}\n", counter, e);
                Ok(())
            }
        },
    }
}
