use std::fmt::Display;

use indent_write::indentable::Indentable;

pub fn print_error<E: std::fmt::Debug>(error: E) -> E {
    eprintln!("Error: {:?}", error);
    error
}

pub fn indent<'a, T: Display>(t: &'a T) -> indent_write::indentable::Indented<'static, &'a T> {
    t.indented("    ")
}

pub fn is_debug_enabled() -> bool {
    if let Some((_, val)) = std::env::vars_os().find(|(k, _)| k.eq("DEBUG")) {
        if val == "1" {
            return true;
        }
    }
    false
}

pub fn is_compilation_enabled() -> bool {
    if let Some((_, val)) = std::env::vars_os().find(|(k, _)| k.eq("COMPILE")) {
        if val == "1" {
            return true;
        }
    }
    false
}
