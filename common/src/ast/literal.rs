use rust_decimal::Decimal;

use super::{expression::Expression, ASTDepth, Identifier};
use crate::util::indent;
use std::{fmt::Display, rc::Rc};

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum Literal {
    Bool(bool),
    Number(Decimal),
    String(String),
    Tuple(TupleItems<Expression>),
    List(TupleItems<Expression>),
    ListCons(Rc<Expression>, Rc<Expression>),
    Struct(Identifier, Rc<StructProps>),
    Map(TupleItems<MapKVPair<Expression>>),
    MapCons(TupleItems<MapKVPair<Expression>>, Rc<Expression>),
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Bool(b) => b.fmt(f),
            Literal::Number(num) => f.write_fmt(format_args!("{num}")),
            Literal::String(string) => f.write_fmt(format_args!("\"{string}\"")),
            Literal::Tuple(items) => {
                if items.ast_depth() > 6 {
                    let items_str = format!("{items}");
                    if items_str.split('\n').count() == 1 {
                        f.write_fmt(format_args!("{{ {items} }}"))
                    } else {
                        f.write_fmt(format_args!("{{\n{}\n}}", indent(items)))
                    }
                } else {
                    f.write_fmt(format_args!("{{ {items} }}"))
                }
            }
            Literal::List(items) => {
                if items.ast_depth() > 6 {
                    let items_str = format!("{items}");
                    if items_str.split('\n').count() == 1 {
                        f.write_fmt(format_args!("[ {items} ]"))
                    } else {
                        f.write_fmt(format_args!("[\n{}\n]", indent(items)))
                    }
                } else {
                    f.write_fmt(format_args!("[ {items} ]"))
                }
            }
            Literal::ListCons(first, rest) => {
                if first.ast_depth() > 6 || rest.ast_depth() > 6 {
                    let first_lines = format!("{first}").lines().count();
                    let rest_lines = format!("{rest}").lines().count();

                    match (first_lines, rest_lines) {
                        (1, 1) => f.write_fmt(format_args!("[ {first} :: {rest} ]")),
                        _ => {
                            f.write_fmt(format_args!("[\n{} :: {}\n]", indent(first), indent(rest)))
                        }
                    }
                } else {
                    f.write_fmt(format_args!("[ {first} :: {rest} ]"))
                }
            }
            Literal::Struct(id, props) => f.write_fmt(format_args!("{id}{{ {props} }}")),
            Literal::Map(kv_pairs) => {
                f.write_str("{ ")?;
                kv_pairs.fmt(f)?;
                f.write_str(" }")
            }
            Literal::MapCons(kv_pairs, rest) => {
                f.write_str("{ ")?;
                kv_pairs.fmt(f)?;
                f.write_str(" :: ")?;
                rest.fmt(f)?;
                f.write_str(" }")
            }
        }
    }
}

impl ASTDepth for Literal {
    fn ast_depth(&self) -> usize {
        match self {
            Literal::Bool(_) => 1,
            Literal::Number(_) => 1,
            Literal::String(_) => 1,
            Literal::Tuple(items) => 1 + items.iter().map(|i| i.ast_depth()).sum::<usize>(),
            Literal::List(items) => 1 + items.iter().map(|i| i.ast_depth()).sum::<usize>(),
            Literal::ListCons(first, rest) => first.ast_depth() + rest.ast_depth(),
            Literal::Struct(_id, props) => {
                1 + props
                    .iter()
                    .map(|(_name, val)| val.ast_depth())
                    .sum::<usize>()
            }
            Literal::Map(kv_pairs) => 1 + kv_pairs.ast_depth(),
            Literal::MapCons(kv_pairs, rest) => 1 + kv_pairs.ast_depth() + rest.ast_depth(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct TupleItems<I> {
    items: Vec<Rc<I>>,
}

impl<I: Display> TupleItems<I> {
    pub fn new(first: I, rest: Vec<I>) -> Self {
        let mut items = Vec::with_capacity(rest.len() + 1);
        items.push(Rc::new(first));
        for item in rest {
            items.push(Rc::new(item))
        }
        Self::from(items)
    }

    pub fn from(items: Vec<Rc<I>>) -> Self {
        TupleItems { items }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> std::slice::Iter<Rc<I>> {
        self.items.iter()
    }
}

impl<I: Display> FromIterator<Rc<I>> for TupleItems<I> {
    fn from_iter<T: IntoIterator<Item = Rc<I>>>(iter: T) -> Self {
        TupleItems::from(iter.into_iter().collect())
    }
}

impl<I: Display + ASTDepth> Display for TupleItems<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indent_items = self.ast_depth() > 100 || self.items.iter().any(|i| i.ast_depth() > 6);
        let mut is_first = true;
        for item in self.items.iter() {
            if !is_first {
                if indent_items {
                    f.write_str(",\n")?;
                } else {
                    f.write_str(", ")?;
                }
            }

            item.fmt(f)?;

            is_first = false;
        }
        Ok(())
    }
}

impl<I: ASTDepth> ASTDepth for TupleItems<I> {
    fn ast_depth(&self) -> usize {
        1 + self.items.iter().map(|i| i.ast_depth()).sum::<usize>()
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct StructProps {
    props: Vec<(Identifier, Rc<Expression>)>,
}

impl StructProps {
    pub fn new(
        first: (Identifier, Rc<Expression>),
        rest: Vec<(Identifier, Rc<Expression>)>,
    ) -> Self {
        let mut boxed_props = Vec::with_capacity(1 + rest.len());
        let (f_id, f_expr) = first;
        boxed_props.push((f_id, f_expr));
        for (id, expr) in rest {
            boxed_props.push((id, expr))
        }
        Self::from(boxed_props)
    }

    pub fn from(props: Vec<(Identifier, Rc<Expression>)>) -> Self {
        StructProps { props }
    }

    pub fn len(&self) -> usize {
        self.props.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> std::slice::Iter<(Identifier, Rc<Expression>)> {
        self.props.iter()
    }
}

impl Display for StructProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str = self.iter().fold(String::from(""), |acc, (id, expr)| {
            if acc.is_empty() {
                format!("{id}: {expr}")
            } else {
                format!("{acc}, {id}: {expr}")
            }
        });

        f.write_fmt(format_args!("{fmt_str}"))
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct MapKVPair<T: Display + ASTDepth> {
    pub key: Rc<T>,
    pub value: Rc<T>,
}

impl<T: Display + ASTDepth> MapKVPair<T> {
    pub fn new(key: Rc<T>, value: Rc<T>) -> Self {
        Self { key, value }
    }

    pub fn pair(&self) -> (Rc<T>, Rc<T>) {
        (Rc::clone(&self.key), Rc::clone(&self.value))
    }
}

impl<T: Display + ASTDepth> Display for MapKVPair<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.key.fmt(f)?;
        f.write_str(" : ")?;
        self.value.fmt(f)
    }
}

impl<T: Display + ASTDepth> ASTDepth for MapKVPair<T> {
    fn ast_depth(&self) -> usize {
        self.key.ast_depth() + self.value.ast_depth()
    }
}

impl<T: Display + ASTDepth> From<(Rc<T>, Rc<T>)> for MapKVPair<T> {
    fn from(pair: (Rc<T>, Rc<T>)) -> Self {
        MapKVPair {
            key: pair.0,
            value: pair.1,
        }
    }
}

impl<T: Display + ASTDepth> From<(T, T)> for MapKVPair<T> {
    fn from(pair: (T, T)) -> Self {
        MapKVPair {
            key: Rc::new(pair.0),
            value: Rc::new(pair.1),
        }
    }
}
