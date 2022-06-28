use std::fmt::Display;

use indent_write::indentable::Indentable;

use self::{
    expression::{Expression, FnDefArgs, LambdaArgs, TupleItems},
    module_def::ModuleExports,
};

pub mod expression;
pub mod fn_call;
pub mod fn_def;
pub mod module_def;

pub type Identifier = String;
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Program {
    nodes: Vec<Box<AST>>,
}

impl Program {
    pub fn new(nodes: Vec<AST>) -> Self {
        Self::from(Vec::from_iter(nodes.iter().map(|d| Box::new(d.clone()))))
    }

    pub fn from(nodes: Vec<Box<AST>>) -> Self {
        Program { nodes: nodes }
    }
}

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str =
            self.nodes
                .iter()
                .map(|ast| format!("{}", ast))
                .fold(String::from(""), |acc, fmt| {
                    if acc == "" {
                        fmt
                    } else {
                        format!("{}\n\n{}", acc, fmt)
                    }
                });

        f.write_fmt(format_args!("{}", fmt_str))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AST {
    RootComment(String),
    FnDef(Identifier, FnDefArgs, Box<Expression>),
    ModuleDef(Identifier, ModuleExports),
    TypeDef(Identifier, Box<TypeExpression>),
}

impl Display for AST {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AST::RootComment(comment) => f.write_fmt(format_args!("//{}", comment)),
            AST::FnDef(id, args, body) => f.write_fmt(format_args!(
                "let {}{} =\n{}",
                id,
                args,
                body.indented("    ")
            )),
            AST::ModuleDef(id, exports) => f.write_fmt(format_args!("module {} ({})", id, exports)),
            AST::TypeDef(id, type_expr) => {
                f.write_fmt(format_args!("type {} :: {}", id, type_expr))
            }
        }
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
                let mut result: Option<std::fmt::Result> = None;
                for (id, type_expr) in property_types {
                    result = Some(f.write_fmt(format_args!("{} :: {}", id, type_expr)))
                }
                return result.unwrap();
            }
        }
    }
}
