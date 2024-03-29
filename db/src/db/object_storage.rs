use std::collections::HashMap;
use std::fmt::Debug;

use super::{id, Identifier};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ObjectStorage {
    ids: HashMap<String, Identifier>,
}

const DEFAULT_PROPS: [&str; 1] = ["props"];

impl Default for ObjectStorage {
    fn default() -> Self {
        ObjectStorage::new()
    }
}

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

    pub fn get_id<S: ToString + ?Sized>(&mut self, key: &S) -> Identifier {
        let id_s = key.to_string();
        match self.ids.get(&id_s) {
            Some(id) => id.to_owned(),
            None => {
                let new_id = id(&id_s);
                self.ids.insert(id_s.clone(), new_id);
                new_id
            }
        }
    }

    // pub fn store_object<DB: Datastore>(
    //     &mut self,
    //     db: &'a DB,
    //     identifier: Identifier,
    //     object: Rc<Object>,
    // ) -> DBResult<Rc<PersistedObject>> {
    //     let vtx = object.vertex();
    //     db.create_vertex(&vtx)?;
    //     let p_obj = PersistedObject::new_boxed(vtx.to_owned(), object.to_owned());
    //     let props_p = self.get_id("object_props");
    //     let json: Json = Json::new(object.value());

    //     db.set_vertex_properties(
    //         VertexPropertyQuery::new(
    //             VertexQuery::Specific(SpecificVertexQuery::single(vtx.id)),
    //             props_p,
    //         ),
    //         identifier,
    //         &json,
    //     )?;

    //     self.define_type("Person", vec![]);

    //     Ok(p_obj)
    // }

    pub fn define_type<ID, Props>(&mut self, ident: ID, prop_types: Props) -> Identifier
    where
        ID: ToString,
        Props: IntoIterator<Item = (String, String)> + Debug,
    {
        // TODO: use proper typedefinition / references in Props instead of String
        let id_ = self.get_id(&ident);
        println!("define_type({:?}, {:?})", ident.to_string(), prop_types);
        id_
    }
}
