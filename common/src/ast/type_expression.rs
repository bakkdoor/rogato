use std::{fmt::Display, rc::Rc};

use crate::util::indent;

use self::super::expression::{LambdaArgs, TupleItems};

use super::{ASTDepth, Identifier};

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct TypeDef {
    id: Identifier,
    type_expr: Rc<TypeExpression>,
}

impl TypeDef {
    pub fn new(id: Identifier, type_expr: Rc<TypeExpression>) -> TypeDef {
        TypeDef { id, type_expr }
    }

    pub fn id(&self) -> Identifier {
        self.id.clone()
    }

    pub fn type_expr(&self) -> Rc<TypeExpression> {
        Rc::clone(&self.type_expr)
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

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum TypeExpression {
    BoolType,
    NumberType,
    StringType,
    TypeRef(Identifier),
    FunctionType(LambdaArgs<TypeExpression>, Rc<TypeExpression>), // args & return type
    TupleType(TupleItems<TypeExpression>),
    ListType(Rc<TypeExpression>),
    StructType(StructTypeProperties),
}

impl Display for TypeExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeExpression::BoolType => f.write_str("Bool"),
            TypeExpression::NumberType => f.write_str("Number"),
            TypeExpression::StringType => f.write_str("String"),
            TypeExpression::TypeRef(id) => f.write_str(id),
            TypeExpression::FunctionType(arg_types, return_type) => {
                arg_types.fmt(f)?;
                f.write_str(" -> ")?;
                return_type.fmt(f)
            }
            TypeExpression::TupleType(element_types) => {
                f.write_str("{ ")?;
                element_types.fmt(f)?;
                f.write_str(" }")
            }
            TypeExpression::ListType(type_expr) => {
                f.write_str("[ ")?;
                type_expr.fmt(f)?;
                f.write_str(" ]")
            }
            TypeExpression::StructType(struct_type_props) => {
                f.write_str("{\n")?;
                indent(struct_type_props).fmt(f)?;
                f.write_str("\n}")
            }
        }
    }
}

impl ASTDepth for TypeExpression {
    fn ast_depth(&self) -> usize {
        match self {
            TypeExpression::BoolType => 1,
            TypeExpression::NumberType => 1,
            TypeExpression::StringType => 1,
            TypeExpression::TypeRef(_) => 1,
            TypeExpression::FunctionType(arg_types, return_type) => {
                1 + arg_types.ast_depth() + return_type.ast_depth()
            }
            TypeExpression::TupleType(el_types) => {
                1 + el_types.iter().map(|t| t.ast_depth()).sum::<usize>()
            }
            TypeExpression::ListType(type_expr) => 1 + type_expr.ast_depth(),
            TypeExpression::StructType(struct_type) => 1 + struct_type.ast_depth(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct StructTypeProperties {
    prop_types: Vec<(Identifier, Rc<TypeExpression>)>,
}

impl StructTypeProperties {
    pub fn new<Props: IntoIterator<Item = (Identifier, Rc<TypeExpression>)>>(props: Props) -> Self {
        let prop_types = props.into_iter().collect();
        StructTypeProperties { prop_types }
    }

    pub fn iter(&self) -> std::slice::Iter<(Identifier, Rc<TypeExpression>)> {
        self.prop_types.iter()
    }

    pub fn len(&self) -> usize {
        self.prop_types.len()
    }

    pub fn is_empty(&self) -> bool {
        self.prop_types.is_empty()
    }
}

impl FromIterator<(Identifier, Rc<TypeExpression>)> for StructTypeProperties {
    fn from_iter<T: IntoIterator<Item = (Identifier, Rc<TypeExpression>)>>(iter: T) -> Self {
        StructTypeProperties::new(iter)
    }
}

impl Display for StructTypeProperties {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut is_first = true;
        for (id, expr) in self.prop_types.iter() {
            if !is_first {
                f.write_str("\n")?;
            }
            id.fmt(f)?;
            f.write_str(" :: ")?;
            expr.fmt(f)?;
            is_first = false;
        }
        Ok(())
    }
}

impl ASTDepth for StructTypeProperties {
    fn ast_depth(&self) -> usize {
        self.prop_types
            .iter()
            .map(|(_id, type_expr)| type_expr.ast_depth())
            .sum::<usize>()
    }
}
