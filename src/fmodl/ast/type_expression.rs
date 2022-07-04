use std::fmt::Display;

use self::super::expression::{LambdaArgs, TupleItems};

use super::Identifier;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TypeExpression {
    IntType,
    StringType,
    TypeRef(Identifier),
    FunctionType(Box<LambdaArgs<TypeExpression>>, Box<TypeExpression>), // args & return type
    TupleType(TupleItems<TypeExpression>),
    ListType(Box<TypeExpression>),
    StructType(Vec<(Identifier, Box<TypeExpression>)>),
}

impl Display for TypeExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeExpression::IntType => f.write_fmt(format_args!("{}", "Int")),
            TypeExpression::StringType => f.write_fmt(format_args!("{}", "String")),
            TypeExpression::TypeRef(id) => f.write_fmt(format_args!("{}", id)),
            TypeExpression::FunctionType(arg_types, return_type) => {
                f.write_fmt(format_args!("{} -> {}", arg_types, return_type))
            }
            TypeExpression::TupleType(element_types) => {
                f.write_fmt(format_args!("{{ {} }}", element_types))
            }
            TypeExpression::ListType(type_expr) => f.write_fmt(format_args!("[ {} ]", type_expr)),
            TypeExpression::StructType(property_types) => {
                let mut result: Option<std::fmt::Result> = None;
                for (id, type_expr) in property_types {
                    result = Some(f.write_fmt(format_args!("{} :: {}", id, type_expr)))
                }
                result.unwrap()
            }
        }
    }
}
