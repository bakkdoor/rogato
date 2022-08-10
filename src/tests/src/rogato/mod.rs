#[cfg(test)]
pub mod db;
#[cfg(test)]
pub mod parser;

#[cfg(test)]
pub mod interpreter;

pub use rogato_parser::{parse, parse_ast, parse_expr};

#[macro_export]
macro_rules! assert_parse {
    ($code:expr, $expected:expr) => {
        assert_eq!(
            rogato_parser::parse($code),
            Ok($expected),
            "Expected program code to parse: {:?}",
            $code
        )
    };
}

#[macro_export]
macro_rules! assert_parse_ast {
    ($code:expr, $expected:expr) => {
        assert_eq!(
            rogato_parser::parse_ast($code),
            Ok($expected),
            "Expected code to parse: {:?}",
            $code
        )
    };
}

#[macro_export]
macro_rules! assert_parse_expr {
    ($code:expr, $expected:expr) => {
        assert_eq!(
            rogato_parser::parse_expr($code),
            Ok($expected),
            "Expected expression code to parse: {:?}",
            $code
        )
    };
}
