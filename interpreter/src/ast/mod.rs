use rogato_common::{
    ast::AST,
    val::{self, ValueRef},
};

use super::{EvalContext, EvalError, Evaluate};

pub mod expression;
pub mod fn_call;
pub mod fn_def;
pub mod lambda;
pub mod let_expression;
pub mod literal;
pub mod module_def;
pub mod program;
pub mod query;
pub mod type_expression;

impl Evaluate<ValueRef> for AST {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        match self {
            AST::RootComment(_) => Ok(val::none()),
            AST::FnDef(fn_def) => fn_def.evaluate(context),
            AST::ModuleDef(mod_def) => mod_def.evaluate(context),
            AST::Use(_id) => Ok(val::none()),
            AST::TypeDef(type_def) => type_def.evaluate(context),
        }
    }
}
