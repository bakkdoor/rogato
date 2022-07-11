use std::collections::HashMap;

use crate::rogato::ast::{fn_def::FnDef, type_expression::TypeDef};

use super::Identifier;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Module {
    fn_defs: HashMap<Identifier, Box<FnDef>>,
    type_defs: HashMap<Identifier, Box<TypeDef>>,
}
