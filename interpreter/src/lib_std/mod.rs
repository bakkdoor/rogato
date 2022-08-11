use crate::environment::Environment;

pub mod math;

pub fn env() -> Environment {
    let mut env = Environment::new();
    let math_mod = math::module();
    env.define_module(math_mod);
    println!("env {:?}", env);
    env
}
