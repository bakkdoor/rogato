use rogato_common::{
    ast::AST,
    val::{self, ValueRef},
};

use crate::environment::{ImportedIdentifier, Imports};

use super::{EvalContext, EvalError, Evaluate};

pub mod expression;
pub mod fn_call;
pub mod fn_def;
pub mod if_else;
pub mod lambda;
pub mod let_expression;
pub mod literal;
pub mod module_def;
pub mod pattern;
pub mod program;
pub mod query;
pub mod type_expression;

impl Evaluate<ValueRef> for AST {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        match self {
            AST::RootComment(_) => Ok(val::none()),
            AST::FnDef(fn_def) => fn_def.borrow().evaluate(context),
            AST::ModuleDef(mod_def) => mod_def.evaluate(context),
            AST::Use(id, imports) => {
                let imports = Imports::Specific(
                    imports
                        .iter()
                        .map(|imp| ImportedIdentifier::Func(imp.clone()))
                        .collect(),
                );
                context.import(id, imports)
            }
            AST::TypeDef(type_def) => type_def.evaluate(context),
        }
    }
}
