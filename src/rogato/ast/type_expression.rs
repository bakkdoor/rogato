use std::fmt::Display;

use crate::rogato::{
    db::{val, Value},
    interpreter::{EvalContext, Evaluate},
    util::indent,
};

use self::super::expression::{LambdaArgs, TupleItems};

use super::{ASTDepth, Identifier};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TypeDef {
    id: Identifier,
    type_expr: Box<TypeExpression>,
}

impl TypeDef {
    pub fn new(id: Identifier, type_expr: Box<TypeExpression>) -> TypeDef {
        TypeDef { id, type_expr }
    }

    pub fn id(&self) -> Identifier {
        self.id.clone()
    }
}

impl Display for TypeDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("type {} :: {}", self.id, self.type_expr))
    }
}

impl ASTDepth for TypeDef {
    fn ast_depth(&self) -> usize {
        1 + self.type_expr.ast_depth()
    }
}

impl<'a> Evaluate<'a, Value> for TypeDef {
    fn evaluate(&self, context: &mut EvalContext<'a>) -> Value {
        val::object(vec![
            ("type", val::string("TypeDef")),
            ("name", val::string(self.id.to_string())),
            ("type_expr", self.type_expr.evaluate(context)),
        ])
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TypeExpression {
    IntType,
    StringType,
    TypeRef(Identifier),
    FunctionType(LambdaArgs<TypeExpression>, Box<TypeExpression>), // args & return type
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
                        if acc.is_empty() {
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

impl ASTDepth for TypeExpression {
    fn ast_depth(&self) -> usize {
        match self {
            TypeExpression::IntType => 1,
            TypeExpression::StringType => 1,
            TypeExpression::TypeRef(_) => 1,
            TypeExpression::FunctionType(arg_types, return_type) => {
                1 + arg_types.ast_depth() + return_type.ast_depth()
            }
            TypeExpression::TupleType(el_types) => {
                1 + el_types.iter().map(|t| t.ast_depth()).sum::<usize>()
            }
            TypeExpression::ListType(type_expr) => 1 + type_expr.ast_depth(),
            TypeExpression::StructType(prop_types) => {
                1 + prop_types
                    .iter()
                    .map(|(_id, type_expr)| type_expr.ast_depth())
                    .sum::<usize>()
            }
        }
    }
}

fn type_ref<ID: ToString>(id: ID) -> Value {
    val::object(vec![
        ("type", val::string("TypeRef")),
        ("name", val::string(id.to_string())),
    ])
}

impl<'a> Evaluate<'a, Value> for TypeExpression {
    fn evaluate(&self, context: &mut EvalContext<'a>) -> Value {
        match self {
            TypeExpression::IntType => type_ref("Int"),
            TypeExpression::StringType => type_ref("String"),
            TypeExpression::TypeRef(id) => type_ref(id),
            TypeExpression::FunctionType(arg_types, return_type) => val::object(vec![
                ("type", val::string("FunctionType")),
                ("args", arg_types.evaluate(context)),
                ("return_type", return_type.evaluate(context)),
            ]),
            TypeExpression::TupleType(el_types) => val::object(vec![
                ("type", val::string("TupleType")),
                ("el_types", el_types.evaluate(context)),
            ]),
            TypeExpression::ListType(type_expr) => val::object(vec![
                ("type", val::string("ListType")),
                ("type_expr", type_expr.evaluate(context)),
            ]),
            TypeExpression::StructType(prop_types) => val::object(vec![
                ("type", val::string("StructType")),
                ("props", prop_types.evaluate(context)),
            ]),
        }
    }
}

impl<'a> Evaluate<'a, Value> for Vec<(Identifier, Box<TypeExpression>)> {
    fn evaluate(&self, context: &mut EvalContext<'a>) -> Value {
        let mut vec = Vec::new();
        for (_id, type_expr) in self.iter() {
            vec.push(type_expr.evaluate(context))
        }
        val::array(vec)
    }
}
