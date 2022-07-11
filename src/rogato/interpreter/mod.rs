use self::environment::Environment;

pub mod environment;
pub mod module;

type Identifier = String;

pub trait Evaluate<T> {
    fn evaluate(&self, env: &mut Environment) -> T;
}
