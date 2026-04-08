use std::rc::Rc;

use rogato_common::ast::{pattern::Pattern, type_expression::TypeExpression, VarIdentifier};

/// Infers the type from a pattern and returns (type, variable_bindings).
/// The variable_bindings map each bound variable to its inferred type.
pub(crate) fn infer_pattern_type(
    pattern: &Pattern,
) -> (TypeExpression, Vec<(VarIdentifier, TypeExpression)>) {
    match pattern {
        Pattern::Var(id) => (
            TypeExpression::Unknown,
            vec![(id.clone(), TypeExpression::Unknown)],
        ),
        Pattern::Number(_) => (TypeExpression::NumberType, vec![]),
        Pattern::Bool(_) => (TypeExpression::BoolType, vec![]),
        Pattern::String(_) => (TypeExpression::StringType, vec![]),
        Pattern::Symbol(_) => (TypeExpression::SymbolType, vec![]),
        Pattern::Any => (TypeExpression::Unknown, vec![]),
        Pattern::EmptyList => (
            TypeExpression::ListType(Rc::new(TypeExpression::Unknown)),
            vec![],
        ),
        Pattern::ListCons(head, tail) => {
            let (head_type, mut bindings) = infer_pattern_type(head);
            let (_tail_type, tail_bindings) = infer_pattern_type(tail);
            bindings.extend(tail_bindings);
            (TypeExpression::ListType(Rc::new(head_type)), bindings)
        }
        Pattern::List(items) => {
            let mut bindings = vec![];
            let mut elem_type = TypeExpression::Unknown;
            for item in items.iter() {
                let (item_type, item_bindings) = infer_pattern_type(item);
                if !matches!(item_type, TypeExpression::Unknown) {
                    elem_type = item_type;
                }
                bindings.extend(item_bindings);
            }
            (TypeExpression::ListType(Rc::new(elem_type)), bindings)
        }
        Pattern::Tuple(_, items) => {
            let mut bindings = vec![];
            for item in items.iter() {
                let (_item_type, item_bindings) = infer_pattern_type(item);
                bindings.extend(item_bindings);
            }
            // Tuple type inference is limited for now
            (TypeExpression::Unknown, bindings)
        }
        Pattern::Map(_) | Pattern::MapCons(_, _) => {
            // Map patterns are complex, just return Unknown for now
            (TypeExpression::Unknown, vec![])
        }
    }
}
