use indradb::VertexPropertyQuery;

use super::{
    object::{Object, PersistedObject},
    DBResult, Datastore, SpecificVertexQuery, VertexQuery,
};

#[allow(dead_code)]
pub fn store_value<DB: Datastore>(db: &DB, object: Box<Object>) -> DBResult<Box<PersistedObject>> {
    let vtx = object.vertex();
    db.create_vertex(&vtx)?;
    let p_obj = PersistedObject::new_boxed(vtx.to_owned(), object.to_owned());
    let props_p = super::id("props");

    db.set_vertex_properties(
        VertexPropertyQuery::new(
            VertexQuery::Specific(SpecificVertexQuery::single(vtx.id)),
            props_p,
        ),
        object.value(),
    )?;

    Ok(p_obj)
}
