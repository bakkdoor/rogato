#[cfg(test)]
pub mod tests;

pub mod codegen;
pub mod error;

use std::rc::Rc;

pub use codegen::{Codegen, CodegenResult};
pub use error::CodegenError;
use inkwell::{context::Context, types::BasicMetadataTypeEnum};
use rogato_common::ast::{
    fn_def::{FnDef, FnDefBody, FnDefVariant},
    pattern::Pattern,
    type_expression::TypeExpression,
};

pub trait LookupType {
    fn to_type<'a, 'ctx>(&self, codegen: Codegen<'a, 'ctx>) -> BasicMetadataTypeEnum<'ctx>;
}

impl LookupType for TypeExpression {
    fn to_type<'a, 'ctx>(&self, codegen: Codegen<'a, 'ctx>) -> BasicMetadataTypeEnum<'ctx> {
        let context: &'ctx Context = codegen.context;
        match self {
            TypeExpression::BoolType => context.bool_type().into(),
            TypeExpression::NumberType => context.f64_type().into(),
            TypeExpression::StringType => unimplemented!("StringType"),
            TypeExpression::TypeRef(type_name) => unimplemented!("TypeRef"),
            TypeExpression::FunctionType(_, _) => unimplemented!("FunctionType"),
            TypeExpression::TupleType(_) => unimplemented!("TupleType"),
            TypeExpression::ListType(_) => unimplemented!("ListType"),
            TypeExpression::StructType(_) => unimplemented!("StructType"),
        }
    }
}

impl LookupType for FnDef {
    fn to_type<'a, 'ctx>(&self, codegen: Codegen<'a, 'ctx>) -> BasicMetadataTypeEnum<'ctx> {
        let context: &'ctx Context = codegen.context;
        let mut arg_types = Vec::new();
        let mut return_type = None;
        for FnDefVariant(args, body) in self.variants_iter() {
            for arg in args.iter() {
                arg_types.push(arg.to_type(codegen));
            }
            return_type = Some(body.to_type(codegen));
        }
        return_type.unwrap()
        // context.
        //     .fn_type(return_type.unwrap().clone(), &arg_types, false)
        //     .into()
    }
}

impl LookupType for Rc<FnDefBody> {
    fn to_type<'a, 'ctx>(&self, codegen: Codegen<'a, 'ctx>) -> BasicMetadataTypeEnum<'ctx> {
        let context: &'ctx Context = codegen.context;
        match &self.as_ref() {
            FnDefBody::NativeFn(func) => unimplemented!("NativeFn LookupType"),
            FnDefBody::RogatoFn(expr) => expr.to_type(codegen),
        }
    }
}

impl LookupType for Rc<Pattern> {
    fn to_type<'a, 'ctx>(&self, codegen: Codegen<'a, 'ctx>) -> BasicMetadataTypeEnum<'ctx> {
        let context: &'ctx Context = codegen.context;
        match &self.as_ref() {
            Pattern::Any => unimplemented!("Any"),
            Pattern::Var(_) => unimplemented!("Variable"),
            Pattern::Tuple(_) => unimplemented!("Tuple"),
            Pattern::List(_) => unimplemented!("List"),
            _ => unimplemented!("Other Pattern"),
        }
    }
}
