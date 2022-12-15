use crate::{EvalContext, Evaluate};
use rogato_common::val::{self};
use rogato_parser::{parse_expr, ParserContext};
use rust_decimal_macros::dec;

#[test]
fn std_math_module() {
    let code_with_vals = [
        ("Std.Math.abs -10", val::number(10)),
        ("Std.Math.abs (10 * -10)", val::number(100)),
        ("Math.abs (100 - 1000)", val::number(900)),
        ("abs -10", val::number(10)),
        ("abs (10 * -10)", val::number(100)),
        ("abs (100 - 1000)", val::number(900)),
        ("Math.sqrt 9", val::some(val::number(3))),
        (
            "sqrt 2",
            val::some(val::number(dec!(1.4142135623730950488016887242))),
        ),
        ("sqrt 4", val::some(val::number(2))),
        ("sqrt 100", val::some(val::number(10))),
        ("sqrt 10000", val::some(val::number(100))),
        ("rescale 420 2", val::number(420)),
        ("rescale 420.69 0", val::number(421)),
        ("rescale 420.691337 2", val::decimal_str("420.69")),
        ("rescale 420.691337 3", val::decimal_str("420.691")),
        ("rescale 420.691337 4", val::decimal_str("420.6913")),
    ];

    let mut eval_ctx = EvalContext::new();
    let parser_ctx = ParserContext::new();

    for (code, val) in code_with_vals.iter() {
        let ast = parse_expr(code, &parser_ctx).unwrap();
        assert_eq!(ast.evaluate(&mut eval_ctx), Ok(val.clone()));
    }
}
