pub use indradb::{
    BulkInsertItem, Datastore, Edge, EdgeDirection, EdgeKey, EdgeProperties, EdgeProperty,
    EdgePropertyQuery, EdgeQuery, EdgeQueryExt, Error, Identifier, PipePropertyPresenceEdgeQuery,
    PipePropertyValueEdgeQuery, PipeVertexQuery, PropertyPresenceEdgeQuery,
    PropertyPresenceVertexQuery, PropertyValueEdgeQuery, PropertyValueVertexQuery,
    RangeVertexQuery, RocksdbDatastore, SpecificVertexQuery, Vertex, VertexPropertyQuery,
    VertexQuery, VertexQueryExt,
};
pub use serde_json::Number;
use std::path::Path;
pub use uuid::Uuid;

pub mod object;
pub mod object_storage;
pub mod val;

pub use object_storage::ObjectStorage;

pub type DBResult<T> = Result<T, indradb::Error>;

#[allow(dead_code)]
pub type ValidationResult<T> = Result<T, indradb::ValidationError>;

pub fn open<P: AsRef<Path>>(db_path: P) -> DBResult<RocksdbDatastore> {
    RocksdbDatastore::new(db_path, None)
}

pub fn id<ID: ToString>(id: ID) -> Identifier {
    Identifier::new(id.to_string())
        .map_err(|e| {
            eprintln!("DB Identifier Error: {:?} : {}", e, id.to_string());
            e
        })
        .unwrap()
}
