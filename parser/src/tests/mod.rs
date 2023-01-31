#[cfg(test)]
pub mod parser;

pub use crate::{parse, parse_ast, parse_expr, ParserContext};

#[macro_export]
macro_rules! assert_parse {
    ($code:expr, $expected:expr) => {
        let parsed = $crate::parse($code, &crate::ParserContext::new());
        let expected = Ok($expected);
        assert_eq!(
            parsed.clone(),
            expected.clone(),
            "\n\nExpected program code to parse:\n{:?}\nExpected:\n{}\nGot:\n{}",
            $code,
            expected.unwrap(),
            parsed.unwrap()
        );
    };
}

#[macro_export]
macro_rules! assert_parse_ast {
    ($code:expr, $expected:expr) => {
        let parsed = $crate::parse_ast($code, &crate::ParserContext::new());
        let expected = Ok($expected);
        assert_eq!(
            parsed.clone(),
            expected.clone(),
            "\n\nExpected code to parse:\n{:?}\nExpected:\n{}\nGot:\n{}",
            $code,
            expected.unwrap(),
            parsed.unwrap()
        );
    };
}

#[macro_export]
macro_rules! assert_parse_expr {
    ($code:expr, $expected:expr) => {
        let parsed = $crate::parse_expr($code, &crate::ParserContext::new());
        let expected = Ok($expected);
        assert_eq!(
            parsed.clone(),
            expected.clone(),
            "\n\nExpected expression code to parse:\n{:?}\nExpected:\n{}\nGot:\n{}",
            $code,
            expected.unwrap(),
            parsed.unwrap()
        );
    };
}
