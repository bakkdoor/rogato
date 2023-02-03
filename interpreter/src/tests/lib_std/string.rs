use crate::{EvalContext, Evaluate};
use rogato_common::val::{self};
use rogato_parser::{parse_expr, ParserContext};

#[test]
fn std_string_module() {
    let code_with_vals = [
        ("String.length \"\"", val::number(0)),
        ("String.length \" \"", val::number(1)),
        ("String.length \"hello\"", val::number(5)),
        ("String.length \"hello, world\"", val::number(12)),
        ("String.reverse \"\"", val::string("")),
        ("String.reverse \"hello\"", val::string("olleh")),
        ("String.reverse \"\twat\"", val::string("taw\t")),
        ("String.split \"\" \",\"", val::list([val::string("")])),
        (
            "String.split \",hello,\" \",\"",
            val::list([val::string(""), val::string("hello"), val::string("")]),
        ),
        (
            "String.split \" hello world! \" \",\"",
            val::list([val::string(" hello world! ")]),
        ),
        (
            "String.split \"hello,world, how ,are,you?\" \",\"",
            val::list([
                val::string("hello"),
                val::string("world"),
                val::string(" how "),
                val::string("are"),
                val::string("you?"),
            ]),
        ),
        ("String.lowercase \"fooBarBaz\"", val::string("foobarbaz")),
        (
            "String.lowercase \" FOOO_baR\t!\"",
            val::string(" fooo_bar\t!"),
        ),
        ("String.uppercase \"fooBarBaz\"", val::string("FOOBARBAZ")),
        (
            "String.uppercase \" FOOO_baR\t!\"",
            val::string(" FOOO_BAR\t!"),
        ),
        ("String.toSymbol \"foo\"", val::symbol("foo")),
        ("String.toSymbol \"foo \"", val::symbol("foo")),
        ("String.toSymbol \" foo\"", val::symbol("foo")),
        ("String.toSymbol \" foo \"", val::symbol("foo")),
        ("String.toSymbol \"fooBar\"", val::symbol("fooBar")),
        ("String.toSymbol \"fooBarBaz\"", val::symbol("fooBarBaz")),
        ("String.toSymbol \" fooBarBaz \"", val::symbol("fooBarBaz")),
    ];

    let mut eval_ctx = EvalContext::new();
    let parser_ctx = ParserContext::new();

    for (code, val) in code_with_vals.iter() {
        let ast = parse_expr(code, &parser_ctx).unwrap();
        assert_eq!(ast.evaluate(&mut eval_ctx), Ok(val.clone()));
    }
}
