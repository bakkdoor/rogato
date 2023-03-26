use indradb::Database;
pub use indradb::{
    BulkInsertItem, Datastore, Edge, EdgeDirection, EdgeProperties, EdgeProperty, Error,
    Identifier, RangeVertexQuery, RocksdbDatastore, SpecificVertexQuery, Vertex,
};
pub use serde_json::Number;
use std::path::Path;
pub use uuid::Uuid;

pub mod object;
pub mod object_storage;

pub use object_storage::ObjectStorage;

pub type DBResult<T> = Result<Database<T>, indradb::Error>;

pub type ValidationResult<T> = Result<T, indradb::ValidationError>;

pub fn open<P: AsRef<Path>>(db_path: P) -> DBResult<RocksdbDatastore> {
    RocksdbDatastore::new_db(db_path)
}

pub fn id<ID: ToString>(id: ID) -> Identifier {
    Identifier::new(id.to_string())
        .map_err(|e| {
            eprintln!("DB Identifier Error: {:?} : {}", e, id.to_string());
            e
        })
        .unwrap()
}
