use std::rc::Rc;

use super::*;
use serde_json::{Map, Value};

#[derive(Clone, Eq, Debug)]
pub struct Object {
    type_id: Identifier,
    props: Map<String, Value>,
}

impl Object {
    pub fn new<Props: IntoIterator<Item = (String, Value)>>(
        type_id: Identifier,
        props: Props,
    ) -> Self {
        Self {
            type_id,
            props: Map::from_iter(props),
        }
    }

    pub fn new_boxed<Props: IntoIterator<Item = (String, Value)>>(
        type_id: Identifier,
        props: Props,
    ) -> Rc<Self> {
        Rc::new(Self::new(type_id, props))
    }

    pub fn vertex(&self) -> Vertex {
        Vertex::new(self.type_id.to_owned())
    }

    pub fn value(&self) -> Value {
        Value::Object(self.props.to_owned())
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        if !self.type_id.eq(&other.type_id) {
            return false;
        }
        if self.props.len() != other.props.len() {
            return false;
        }

        for (key, val) in self.props.iter() {
            if !other.props.contains_key(key) {
                return false;
            }

            match other.props.get(key) {
                Some(other_val) => {
                    if !val.eq(other_val) {
                        return false;
                    }
                    continue;
                }
                None => {
                    return false;
                }
            }
        }

        true
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PersistedObject {
    vertex: Vertex,
    object: Rc<Object>,
}

impl PersistedObject {
    pub fn new(vertex: Vertex, object: Rc<Object>) -> PersistedObject {
        PersistedObject { vertex, object }
    }
    pub fn new_boxed(vertex: Vertex, object: Rc<Object>) -> Rc<PersistedObject> {
        Rc::new(Self::new(vertex, object))
    }
}
