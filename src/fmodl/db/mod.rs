use indradb::{
    Datastore, Edge, EdgeQuery, Identifier, PropertyValueEdgeQuery, RangeVertexQuery,
    RocksdbDatastore,
};
use serde_json::Number;
use std::{fmt::Debug, path::Path};
use uuid::Uuid;
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

fn print_error<S: ToString, E: Debug>(prop_name: S, error: E) -> E {
    eprintln!(
        "Failed to index DB property {:?} : {:?}",
        prop_name.to_string(),
        error
    );
    error
}

pub fn create_vertex<DB: Datastore, S: Into<String>>(db: &DB, id_s: S) -> DBResult<Uuid> {
    let id_string = id_s.into();
    match Identifier::new(id_string.to_owned()) {
        Ok(id) => db.create_vertex_from_type(id),
        Err(error) => {
            print_error(id_string, error);
            Err(indradb::Error::Unsupported)
        }
    }
}

pub fn do_stuff<DB: Datastore + Debug>(db: &DB) {
    println!("DB: do stuff with {:?}", db);
    index_prop(db, "testid").unwrap();
    let res1 = query_prop(db, "testid", val::string("test-id-value")).unwrap();
    let res2 = query_prop(db, "testid", val::number(Number::from(8540i32))).unwrap();
    let res3 = query_prop(db, "testid", val::number(Number::from(8540i64))).unwrap();
    println!("res1 {:?}", res1);
    println!("res2 {:?}", res2);
    println!("res3 {:?}", res3);
    let res4 = create_vertex(db, "testid").unwrap();
    println!("res4 {:?}", res4);
    let vertex_type_id = Identifier::new(String::from("testid")).unwrap();
    let res5 = db
        .get_vertices(RangeVertexQuery::new().t(vertex_type_id).into())
        .unwrap();
    println!("res5 {:?}", res5);
}
