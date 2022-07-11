use self::environment::Environment;

pub mod environment;
pub mod module;

type Identifier = String;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EvalContext<'a> {
    env: Environment<'a>,
}

impl<'a> EvalContext<'a> {
    #[allow(dead_code)]
    pub fn new() -> EvalContext<'a> {
        EvalContext {
            env: Environment::new(),
        }
    }

    #[allow(dead_code)]
    pub fn from_env(env: &Environment<'a>) -> EvalContext<'a> {
        EvalContext { env: env.clone() }
    }
}

pub trait Evaluate<'a, T> {
    fn evaluate(&self, context: &mut EvalContext<'a>) -> T;
}
