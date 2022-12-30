use super::{invalid_args, op_fn, with_number_op_args};
use crate::module::Module;
use rogato_common::{
    ast::module_def::ModuleExports,
    val::{self, Value},
};
use rust_decimal::{prelude::ToPrimitive, Decimal, MathematicalOps};

pub fn module() -> Module {
    let mut module = Module::new("Std.Math");
    module.export(&ModuleExports::new(vec![
        "+".into(),
        "-".into(),
        "*".into(),
        "/".into(),
        "%".into(),
        "^".into(),
        "abs".into(),
        "round".into(),
        "ceil".into(),
        "floor".into(),
        "trunc".into(),
        "fract".into(),
        "rescale".into(),
        "max".into(),
        "min".into(),
        "sqrt".into(),
        "isEven".into(),
        "isOdd".into(),
    ]));

    module.fn_def(
        "+",
        op_fn(move |_ctx, args| with_number_op_args("+", args, |a, b| Ok(a.saturating_add(b)))),
    );

    module.fn_def(
        "-",
        op_fn(move |_ctx, args| with_number_op_args("-", args, |a, b| Ok(a.saturating_sub(b)))),
    );

    module.fn_def(
        "*",
        op_fn(move |_ctx, args| with_number_op_args("*", args, |a, b| Ok(a.saturating_mul(b)))),
    );

    module.fn_def(
        "/",
        op_fn(move |_ctx, args| {
            with_number_op_args("/", args, |a, b| {
                a.checked_div(b).map_or(Err(invalid_args("/")), Ok)
            })
        }),
    );

    module.fn_def(
        "%",
        op_fn(move |_ctx, args| {
            with_number_op_args("%", args, |a, b| {
                a.checked_rem(b).map_or(Err(invalid_args("%")), Ok)
            })
        }),
    );

    module.fn_def(
        "^",
        op_fn(move |_ctx, args| {
            with_number_op_args("^", args, |a, b| {
                a.checked_powd(b).map_or(Err(invalid_args("^")), Ok)
            })
        }),
    );

    module.fn_def_native("abs", &["num"], move |_ctx, args| {
        let error = Err(invalid_args("abs"));
        match args.get(0) {
            Some(val) => match &**val {
                Value::Number(num) => Ok(val::number(num.abs())),
                _ => error,
            },
            _ => error,
        }
    });

    module.fn_def_native("round", &["num"], move |_ctx, args| {
        let error = Err(invalid_args("round"));
        match args.get(0) {
            Some(val) => match &**val {
                Value::Number(num) => Ok(val::number(num.round())),
                _ => error,
            },
            _ => error,
        }
    });

    module.fn_def_native("ceil", &["num"], move |_ctx, args| {
        let error = Err(invalid_args("ceil"));
        match args.get(0) {
            Some(val) => match &**val {
                Value::Number(num) => Ok(val::number(num.ceil())),
                _ => error,
            },
            _ => error,
        }
    });

    module.fn_def_native("floor", &["num"], move |_ctx, args| {
        let error = Err(invalid_args("floor"));
        match args.get(0) {
            Some(val) => match &**val {
                Value::Number(num) => Ok(val::number(num.floor())),
                _ => error,
            },
            _ => error,
        }
    });

    module.fn_def_native("trunc", &["num"], move |_ctx, args| {
        let error = Err(invalid_args("trunc"));
        match args.get(0) {
            Some(val) => match &**val {
                Value::Number(num) => Ok(val::number(num.trunc())),
                _ => error,
            },
            _ => error,
        }
    });

    module.fn_def_native("fract", &["num"], move |_ctx, args| {
        let error = Err(invalid_args("fract"));
        match args.get(0) {
            Some(val) => match &**val {
                Value::Number(num) => Ok(val::number(num.fract())),
                _ => error,
            },
            _ => error,
        }
    });

    module.fn_def_native("rescale", &["num", "scale"], move |_ctx, args| {
        let error = Err(invalid_args("rescale"));
        match (args.len(), args.get(0), args.get(1)) {
            (2, Some(val), Some(scale)) => match (&**val, &**scale) {
                (Value::Number(num), Value::Number(scale)) => match scale.floor().to_u32() {
                    Some(scale) => {
                        let mut result = Decimal::clone(num);
                        result.rescale(scale);
                        Ok(val::number(result))
                    }
                    None => error,
                },
                _ => error,
            },
            _ => error,
        }
    });

    module.fn_def_native("max", &["a", "b"], move |_ctx, args| {
        let error = Err(invalid_args("max"));
        match (args.get(0), args.get(1)) {
            (Some(a), Some(b)) => match (&**a, &**b) {
                (Value::Number(ad), Value::Number(bd)) => Ok(val::number((*ad).max(*bd))),
                _ => error,
            },
            _ => error,
        }
    });

    module.fn_def_native("min", &["a", "b"], move |_ctx, args| {
        let error = Err(invalid_args("min"));
        match (args.get(0), args.get(1)) {
            (Some(a), Some(b)) => match (&**a, &**b) {
                (Value::Number(ad), Value::Number(bd)) => Ok(val::number((*ad).min(*bd))),
                _ => error,
            },
            _ => error,
        }
    });

    module.fn_def_native("sqrt", &["num"], move |_ctx, args| {
        let error = Err(invalid_args("srqt"));
        match args.get(0) {
            Some(a) => match &**a {
                Value::Number(num) => Ok(val::option(num.sqrt().map(val::number))),
                _ => error,
            },
            _ => error,
        }
    });

    module.fn_def_native("isEven", &["num"], move |_ctx, args| {
        let error = Err(invalid_args("isEven"));
        match args.get(0) {
            Some(a) => match &**a {
                Value::Number(num) => Ok(val::bool(
                    num.checked_rem(2.into())
                        .map(|x| x == 0.into())
                        .unwrap_or(false),
                )),
                _ => error,
            },
            _ => error,
        }
    });

    module.fn_def_native("isOdd", &["num"], move |_ctx, args| {
        let error = Err(invalid_args("isOdd"));
        match args.get(0) {
            Some(a) => match &**a {
                Value::Number(num) => Ok(val::bool(
                    num.checked_rem(2.into())
                        .map(|x| x == 1.into())
                        .unwrap_or(false),
                )),
                _ => error,
            },
            _ => error,
        }
    });

    module
}
