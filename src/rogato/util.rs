use std::fmt::Display;

use indent_write::indentable::Indentable;

pub fn print_error<E: std::fmt::Debug>(error: E) -> E {
    eprintln!("Error: {:?}", error);
    error
}

pub fn indent<T: Display>(t: T) -> indent_write::indentable::Indented<'static, T> {
    t.indented("    ")
}

pub fn prepend_vec<T>(first: T, rest: &mut Vec<T>) -> Vec<T> {
    let mut joined = Vec::new();
    joined.push(first);
    joined.append(rest);
    joined
}
