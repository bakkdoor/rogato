pub use indradb::{
    BulkInsertItem, Datastore, Edge, EdgeKey, EdgePropertyQuery, EdgeQuery, Identifier,
    PropertyValueEdgeQuery, RangeVertexQuery, RocksdbDatastore, Vertex,
};
#[allow(unused_imports)]
use indradb::{
    EdgeDirection, PipePropertyPresenceEdgeQuery, PipePropertyValueEdgeQuery, PipeVertexQuery,
    PropertyPresenceEdgeQuery, PropertyPresenceVertexQuery, PropertyValueVertexQuery,
    VertexPropertyQuery, VertexQuery,
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
    let friendship_edge_id = id("FriendShip");
    let bff_tag_id = id("bff");

    db.index_property(name_prop_id.to_owned())?;
    db.index_property(age_prop_id.to_owned())?;
    db.index_property(bff_tag_id.clone())?;

    for i in 0..1000 {
        let id1 = db.create_vertex_from_type(person_type_id.to_owned())?;
        let id2 = db.create_vertex_from_type(person_type_id.to_owned())?;

        let friendship_edge_key =
            EdgeKey::new(id1.clone(), friendship_edge_id.clone(), id2.clone());

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
                val::number(i * 1000),
            ),
            indradb::BulkInsertItem::VertexProperty(
                id2.clone(),
                age_prop_id.clone(),
                val::number(i * 9999),
            ),
            indradb::BulkInsertItem::Edge(friendship_edge_key.clone()),
            indradb::BulkInsertItem::EdgeProperty(
                friendship_edge_key.clone(),
                bff_tag_id.clone(),
                val::bool(true),
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
    ))?;
    println!(
        "vertices_with_name query results: {}",
        vertices_with_name.len()
    );

    let vertex_props = db.get_vertex_properties(VertexPropertyQuery::new(
        VertexQuery::PropertyPresence(PropertyPresenceVertexQuery::new(age_prop_id.clone())),
        name_prop_id.clone(),
    ))?;
    println!(
        "vertex_props query (age presence) result count: {}",
        vertex_props.len()
    );

    let edge_query =
        EdgeQuery::PropertyPresence(PropertyPresenceEdgeQuery::new(bff_tag_id.clone()));

    let vertex_props = db.get_vertex_properties(VertexPropertyQuery::new(
        VertexQuery::Pipe(PipeVertexQuery::new(
            Box::new(edge_query),
            EdgeDirection::Inbound,
        )),
        name_prop_id.clone(),
    ))?;
    println!(
        "vertex_props query (age presence, incoming Friendship edge) result count: {}",
        vertex_props.len()
    );

    Ok(())
}
