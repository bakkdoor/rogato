pub use indradb::{
    BulkInsertItem, Datastore, Edge, EdgeKey, EdgePropertyQuery, EdgeQuery, Identifier,
    PropertyValueEdgeQuery, RangeVertexQuery, RocksdbDatastore, Vertex,
};
use indradb::{
    PropertyPresenceVertexQuery, PropertyValueVertexQuery, VertexPropertyQuery, VertexQuery,
};
pub use serde_json::Number;
use std::{fmt::Debug, path::Path, str::FromStr};
pub use uuid::Uuid;

pub mod val;

pub type DBResult<T> = Result<T, indradb::Error>;

#[allow(dead_code)]
pub type ValidationResult<T> = Result<T, indradb::ValidationError>;

pub fn open<P: AsRef<Path>>(db_path: P) -> DBResult<RocksdbDatastore> {
    RocksdbDatastore::new(db_path, None)
}

pub fn id(id: &str) -> Identifier {
    Identifier::from_str(id).unwrap()
}

#[allow(dead_code)]
fn print_error<S: ToString, E: Debug>(prop_name: S, error: E) -> E {
    eprintln!(
        "Failed to index DB property {:?} : {:?}",
        prop_name.to_string(),
        error
    );
    error
}

pub fn do_stuff<DB: Datastore + Debug>(db: &DB) -> DBResult<()> {
    println!("DB: do stuff with {:?}", db);
    let person_type_id = id("Person");
    let name_prop_id = id("name");
    let age_prop_id = id("name");

    db.index_property(name_prop_id.to_owned())?;
    db.index_property(age_prop_id.to_owned())?;

    for i in 0..10000 {
        let id1 = db.create_vertex_from_type(person_type_id.to_owned())?;
        let id2 = db.create_vertex_from_type(person_type_id.to_owned())?;

        db.bulk_insert(vec![
            indradb::BulkInsertItem::VertexProperty(
                id1.clone(),
                name_prop_id.clone(),
                val::string(format!("John Connor {}", i)),
            ),
            indradb::BulkInsertItem::VertexProperty(
                id2.clone(),
                name_prop_id.clone(),
                val::string(format!("John Connor {}", i)),
            ),
            indradb::BulkInsertItem::VertexProperty(
                id1.clone(),
                age_prop_id.clone(),
                val::number(Number::from(i * 1000)),
            ),
            indradb::BulkInsertItem::VertexProperty(
                id2.clone(),
                age_prop_id.clone(),
                val::number(Number::from(i * 9999)),
            ),
        ])?;
    }

    let vertices = db.get_vertices(RangeVertexQuery::new().t(person_type_id.to_owned()).into())?;
    println!(
        "Vertex query for type {} gave {} results",
        person_type_id.to_owned().as_str(),
        vertices.len()
    );

    let vertices_with_name = db.get_vertices(indradb::VertexQuery::PropertyValue(
        PropertyValueVertexQuery::new(name_prop_id.clone(), val::string("John Connor 1")),
    ));
    println!(
        "vertices_with_name query results:\n{:?}",
        vertices_with_name
    );

    let vertex_props = db.get_vertex_properties(VertexPropertyQuery::new(
        VertexQuery::PropertyPresence(PropertyPresenceVertexQuery::new(age_prop_id.clone())),
        name_prop_id.clone(),
    ))?;
    println!("vertex_props query result count: {}", vertex_props.len());

    Ok(())
}
