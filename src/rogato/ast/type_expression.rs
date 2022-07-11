use std::fmt::Display;

use crate::rogato::util::indent;

use self::super::expression::{LambdaArgs, TupleItems};

use super::Identifier;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TypeDef {
    id: Identifier,
    type_expr: Box<TypeExpression>,
}

impl TypeDef {
    pub fn new(id: Identifier, type_expr: Box<TypeExpression>) -> TypeDef {
        TypeDef {
            id: id,
            type_expr: type_expr,
        }
    }
}

impl Display for TypeDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("type {} :: {}", self.id, self.type_expr))
    }
}

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
                let fmt_str = property_types
                    .iter()
                    .map(|(id, expr)| format!("{} :: {}", id, expr))
                    .fold(String::from(""), |acc, fmt| {
                        if acc == "" {
                            fmt
                        } else {
                            format!("{}\n{}", acc, fmt)
                        }
                    });

                f.write_fmt(format_args!("{{\n{}\n}}", indent(fmt_str)))
            }
        }
    }
}
