use std::fmt::Display;

use indent_write::indentable::Indentable;

#[allow(dead_code)]
pub fn print_error<E: std::fmt::Debug>(error: E) -> E {
    eprintln!("Error: {:?}", error);
    error
}

pub fn indent<T: Display>(t: T) -> indent_write::indentable::Indented<'static, T> {
    t.indented("    ")
}
