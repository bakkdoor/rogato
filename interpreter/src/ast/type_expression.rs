use std::rc::Rc;

use crate::{EvalContext, EvalError, Evaluate, Identifier, ValueRef};
use rogato_common::{
    ast::type_expression::{TypeDef, TypeExpression},
    val,
};

impl Evaluate<ValueRef> for TypeDef {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        Ok(val::object(vec![
            ("type", val::string("TypeDef")),
            ("name", val::string(self.id().to_string())),
            ("type_expr", self.type_expr().evaluate(context)?),
        ]))
    }
}

fn type_ref<ID: ToString>(id: ID) -> ValueRef {
    val::object(vec![
        ("type", val::string("TypeRef")),
        ("name", val::string(id.to_string())),
    ])
}

impl Evaluate<ValueRef> for TypeExpression {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        Ok(match self {
            TypeExpression::IntType => type_ref("Int"),
            TypeExpression::StringType => type_ref("String"),
            TypeExpression::TypeRef(id) => type_ref(id),
            TypeExpression::FunctionType(arg_types, return_type) => val::object(vec![
                ("type", val::string("FunctionType")),
                ("args", arg_types.evaluate(context)?),
                ("return_type", return_type.evaluate(context)?),
            ]),
            TypeExpression::TupleType(el_types) => val::object(vec![
                ("type", val::string("TupleType")),
                ("el_types", el_types.evaluate(context)?),
            ]),
            TypeExpression::ListType(type_expr) => val::object(vec![
                ("type", val::string("ListType")),
                ("type_expr", type_expr.evaluate(context)?),
            ]),
            TypeExpression::StructType(prop_types) => val::object(vec![
                ("type", val::string("StructType")),
                ("props", prop_types.evaluate(context)?),
            ]),
        })
    }
}

impl Evaluate<ValueRef> for Vec<(Identifier, Rc<TypeExpression>)> {
    fn evaluate(&self, context: &mut EvalContext) -> Result<ValueRef, EvalError> {
        let mut vec = Vec::new();
        for (_id, type_expr) in self.iter() {
            vec.push(type_expr.evaluate(context)?)
        }
        Ok(val::list(vec))
    }
}