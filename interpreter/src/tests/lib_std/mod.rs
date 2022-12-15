#[cfg(test)]
pub mod list;

#[cfg(test)]
pub mod map;

#[cfg(test)]
pub mod math;

#[cfg(test)]
pub mod string;

use crate::{EvalContext, Evaluate};
use rogato_common::val::{self};
use rogato_parser::{parse_expr, ParserContext};

#[test]
fn std() {
    let code_with_vals = [
        (
            "Std.range 10",
            val::list([
                val::number(0),
                val::number(1),
                val::number(2),
                val::number(3),
                val::number(4),
                val::number(5),
                val::number(6),
                val::number(7),
                val::number(8),
                val::number(9),
            ]),
        ),
        (
            "Std.range 1 10",
            val::list([
                val::number(1),
                val::number(2),
                val::number(3),
                val::number(4),
                val::number(5),
                val::number(6),
                val::number(7),
                val::number(8),
                val::number(9),
            ]),
        ),
        ("Std.random 0", val::number(0)),
        ("random 0", val::number(0)),
        ("random 0 0", val::number(0)),
        ("random 1 1", val::number(1)),
        ("random 1.0 1", val::number(1)),
        ("random 1 1.0", val::number(1)),
        ("random 1.0 1.0", val::number(1)),
        ("toString 1", val::string("1")),
        ("toString 1.0", val::string("1.0")),
        ("toString 1.1", val::string("1.1")),
        ("toString 1.5987", val::string("1.5987")),
        ("toString {1.0, \"foo\"}", val::string("{ 1.0, foo }")),
        ("toString \"foo bar baz\"", val::string("foo bar baz")),
        (
            "toString (\"foo\" ++ \" \" ++ \"bar\")",
            val::string("foo bar"),
        ),
    ];

    let mut eval_ctx = EvalContext::new();
    let parser_ctx = ParserContext::new();

    for (code, val) in code_with_vals.iter() {
        let ast = parse_expr(code, &parser_ctx).unwrap();
        assert_eq!(ast.evaluate(&mut eval_ctx), Ok(val.clone()));
    }
}
