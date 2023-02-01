use indent_write::indentable::Indentable;
use std::fmt::Display;

pub fn print_error<E: std::fmt::Debug>(error: E) -> E {
    eprintln!("Error: {error:?}");
    error
}

pub fn indent<T: Display>(t: &T) -> indent_write::indentable::Indented<'static, &T> {
    t.indented("    ")
}

pub fn is_debug_enabled() -> bool {
    is_envar_set("DEBUG")
}

pub fn is_compilation_enabled() -> bool {
    is_envar_set("COMPILE")
}

fn is_envar_set(envar: &str) -> bool {
    if let Some((_, val)) = std::env::vars_os().find(|(k, _)| k.eq(envar)) {
        if val == "1" {
            return true;
        }
    }
    false
}

#[macro_export]
macro_rules! flame_guard {
    ($fmt:expr, $($rest:expr),+) => {
        #[cfg(feature = "flame_it")]
        let _guard = flame::start_guard(format!($fmt, $($rest),*));
    };
}
