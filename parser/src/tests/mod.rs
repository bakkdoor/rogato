#[cfg(test)]
pub mod parser;

pub use crate::{parse, parse_ast, parse_expr, ParserContext};

#[macro_export]
macro_rules! assert_parse {
    ($code:expr, $expected:expr) => {
        assert_eq!(
            crate::parse($code, &crate::ParserContext::new()),
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
            crate::parse_ast($code, &crate::ParserContext::new()),
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
            crate::parse_expr($code, &crate::ParserContext::new()),
            Ok($expected),
            "Expected expression code to parse: {:?}",
            $code
        )
    };
}
