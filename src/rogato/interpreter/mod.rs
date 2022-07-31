pub mod environment;
pub mod eval_context;
pub mod module;
pub mod native_fn;

pub use eval_context::EvalContext;

type Identifier = String;

pub trait Evaluate<T> {
    fn evaluate(&self, context: &mut EvalContext) -> T;
}
