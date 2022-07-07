pub use indradb::{
    BulkInsertItem, Datastore, Edge, EdgeDirection, EdgeKey, EdgeProperties, EdgeProperty,
    EdgePropertyQuery, EdgeQuery, EdgeQueryExt, Identifier, PipePropertyPresenceEdgeQuery,
    PipePropertyValueEdgeQuery, PipeVertexQuery, PropertyPresenceEdgeQuery,
    PropertyPresenceVertexQuery, PropertyValueEdgeQuery, PropertyValueVertexQuery,
    RangeVertexQuery, RocksdbDatastore, Vertex, VertexPropertyQuery, VertexQuery, VertexQueryExt,
};
pub use serde_json::Number;
use std::{path::Path, str::FromStr};
pub use uuid::Uuid;

pub mod val;

pub type DBResult<T> = Result<T, indradb::Error>;

#[allow(dead_code)]
pub type ValidationResult<T> = Result<T, indradb::ValidationError>;

pub fn open<P: AsRef<Path>>(db_path: P) -> DBResult<RocksdbDatastore> {
    RocksdbDatastore::new(db_path, None)
}

pub fn id(id: &str) -> Identifier {
    Identifier::from_str(id)
        .map_err(|e| {
            eprintln!("DB Identifier Error: {:?} : {}", e, id);
            e
        })
        .unwrap()
}
