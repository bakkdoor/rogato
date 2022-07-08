use std::collections::HashMap;

use indradb::VertexPropertyQuery;

use super::{
    id,
    object::{Object, PersistedObject},
    DBResult, Datastore, Identifier, SpecificVertexQuery, VertexQuery,
};

pub struct ObjectStorage {
    ids: HashMap<String, Identifier>,
}

const DEFAULT_PROPS: [&str; 1] = ["props"];

impl ObjectStorage {
    pub fn new() -> Self {
        let props: Vec<(String, Identifier)> = DEFAULT_PROPS
            .iter()
            .map(|p| (p.to_string(), id(p)))
            .collect();

        Self {
            ids: HashMap::from_iter(props),
        }
    }

    pub fn get_id<S: ToString>(&mut self, key: S) -> Identifier {
        let id_s = key.to_string();
        match self.ids.get(&id_s) {
            Some(id) => id.to_owned(),
            None => {
                let new_id = id(&id_s);
                self.ids.insert(id_s.clone(), new_id.clone());
                new_id
            }
        }
    }

    #[allow(dead_code)]
    pub fn store_object<DB: Datastore>(
        &mut self,
        db: &DB,
        object: Box<Object>,
    ) -> DBResult<Box<PersistedObject>> {
        let vtx = object.vertex();
        db.create_vertex(&vtx)?;
        let p_obj = PersistedObject::new_boxed(vtx.to_owned(), object.to_owned());
        let props_p = self.get_id("object_props");

        db.set_vertex_properties(
            VertexPropertyQuery::new(
                VertexQuery::Specific(SpecificVertexQuery::single(vtx.id)),
                props_p,
            ),
            object.value(),
        )?;

        Ok(p_obj)
    }
}
