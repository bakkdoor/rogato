pub use indradb::{
    Datastore, Edge, EdgeQuery, Identifier, PropertyValueEdgeQuery, RangeVertexQuery,
    RocksdbDatastore,
};
use indradb::{EdgeKey, EdgePropertyQuery};
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
    let vertex_type_id = id("testid");
    let name_prop_id = id("name");

    db.index_property(name_prop_id.to_owned())?;

    for _ in 0..10000 {
        let id1 = db.create_vertex_from_type(vertex_type_id.to_owned())?;
        let id2 = db.create_vertex_from_type(vertex_type_id.to_owned())?;
        db.create_edge(&EdgeKey::new(id1, vertex_type_id.to_owned(), id2))?;
        db.set_edge_properties(
            EdgePropertyQuery::new(
                EdgeQuery::PropertyValue(PropertyValueEdgeQuery::new(
                    name_prop_id.to_owned(),
                    val::string("John Connor"),
                )),
                name_prop_id.to_owned(),
            ),
            val::string("John Connor"),
        )?;
    }

    let vertices = db.get_vertices(RangeVertexQuery::new().t(vertex_type_id.to_owned()).into())?;
    let edges = db.get_edges(EdgeQuery::PropertyValue(PropertyValueEdgeQuery::new(
        name_prop_id.to_owned(),
        val::string("John Connor"),
    )));

    println!(
        "Vertex query for type {} gave {} results",
        vertex_type_id.to_owned().as_str(),
        vertices.len()
    );

    println!("edges query results {:?}", edges);

    Ok(())
}
