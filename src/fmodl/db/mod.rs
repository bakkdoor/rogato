use indradb::{Datastore, Edge, EdgeQuery, Identifier, PropertyValueEdgeQuery, RocksdbDatastore};
use std::path::Path;

type DB = RocksdbDatastore;
type DBResult<T> = Result<T, indradb::Error>;

pub fn open<P: AsRef<Path>>(db_path: P) -> DBResult<DB> {
    RocksdbDatastore::new(db_path, None)
}

pub fn with_db<P: AsRef<Path>, T>(db_path: P, f: fn(db: DB) -> DBResult<T>) -> DBResult<T> {
    match open(db_path) {
        Ok(db) => f(db),
        Err(error) => {
            println!("Error: {:?}", error);
            Err(error)
        }
    }
}

pub fn query_prop(db: DB, name: &str, val: serde_json::Value) -> DBResult<Vec<Edge>> {
    match Identifier::new(name) {
        Ok(id) => {
            let query = EdgeQuery::PropertyValue(PropertyValueEdgeQuery::new(id, val));
            db.get_edges(query)
        }
        Err(error) => {
            println!("Failed to create id with: {} Error: {}", name, error);
            Ok(vec![])
        }
    }
}
