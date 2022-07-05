use indradb::{Datastore, Edge, EdgeQuery, Identifier, PropertyValueEdgeQuery, RocksdbDatastore};
use std::{fmt::Debug, path::Path};
pub mod val;

type DBResult<T> = Result<T, indradb::Error>;

pub fn open<P: AsRef<Path>>(db_path: P) -> DBResult<RocksdbDatastore> {
    RocksdbDatastore::new(db_path, None)
}

pub fn query_prop<DB: Datastore>(
    db: &DB,
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

pub fn index_prop<DB: Datastore>(db: &DB, prop_name: &str) -> DBResult<()> {
    match Identifier::new(prop_name) {
        Ok(id) => match db.index_property(id) {
            Ok(()) => Ok(()),
            error => print_error(prop_name, error),
        },
        Err(error) => {
            print_error(prop_name, error);
            Err(indradb::Error::NotIndexed)
        }
    }
}

fn print_error<E: Debug>(prop_name: &str, error: E) -> E {
    eprintln!("Failed to index DB property {:?} : {:?}", prop_name, error);
    error
}

pub fn do_stuff<DB: Datastore + Debug>(db: &DB) {
    println!("DB: do stuff with {:?}", db);
    index_prop(db, "testid").unwrap();
    query_prop(db, "testid", val::string("test-id-value")).unwrap();
}
