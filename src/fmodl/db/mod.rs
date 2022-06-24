use indradb::{Datastore, Edge, EdgeQuery, Identifier, PropertyValueEdgeQuery, RocksdbDatastore};
use std::path::Path;

type DBResult<T> = Result<T, indradb::Error>;

pub fn open<P: AsRef<Path>>(db_path: P) -> DBResult<RocksdbDatastore> {
    RocksdbDatastore::new(db_path, None)
}

pub fn query_prop<DB: Datastore>(
    db: DB,
    name: &str,
    val: serde_json::Value,
) -> DBResult<Vec<Edge>> {
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
