use crate::environment::Environment;

pub mod math;

pub fn env() -> Environment {
    math::env()
}
