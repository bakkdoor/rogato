use crate::rogato::db::val::Value;

pub trait NativeFn: Fn(Vec<Value>) -> Value {
    fn call(&self, args: Vec<Value>) -> Value;
}

impl<F: Clone + 'static> NativeFn for F
where
    F: Fn(Vec<Value>) -> Value,
{
    fn call(&self, args: Vec<Value>) -> Value {
        self(args)
    }
}
