use crate::{EvalContext, Evaluate};
use rogato_parser::{parse, ParserContext};
use std::{env, fs::File, io::Read, path::Path};

#[cfg(test)]
pub mod fn_def;
#[cfg(test)]
pub mod interpreter;
#[cfg(test)]
pub mod std_lib;

fn parse_eval_std(std_mod_name: &str, parser_ctx: &ParserContext, eval_ctx: &mut EvalContext) {
    let curr_dir = env::current_dir().unwrap();
    let root_path = curr_dir
        .as_path()
        .parent()
        .expect("Parent directory expected to be root");
    let file_name = format!("lib/Std/{}.roga", std_mod_name);
    let file_path = root_path.join(Path::new(file_name.as_str()));
    let mut file = File::open(&file_path)
        .expect(format!("Std lib file should exist: {:?}", file_path).as_str());

    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();

    parse(buf.as_str(), parser_ctx)
        .expect(format!("Expected file to parse: {}", file_name).as_str())
        .evaluate(eval_ctx)
        .expect(format!("Expected file to evaluate: {}", file_name).as_str());
}
