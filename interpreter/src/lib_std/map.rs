use super::fn_def;
use crate::module::Module;
use rogato_common::val::{self, Value, ValueRef};

pub fn module() -> Module {
    let mut module = Module::new("Std.Map");

    module.fn_def(fn_def("new", &[], move |_ctx, _args| {
        Ok(ValueRef::new(Value::Map(val::Map::new())))
    }));

    module
}