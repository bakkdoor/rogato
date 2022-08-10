#[allow(unused_imports)]
use rogato_common::{db, db::ObjectStorage};
// use std::path::Path;

// const TEST_DB_PATH: &str = "./rogato.test.db";

// #[test]
// fn store_object() {
//     let datastore = db::open(Path::new(TEST_DB_PATH)).unwrap();
//     let person_t = db::id("Person");

//     let object = db::object::Object::new_boxed(
//         person_t,
//         vec![
//             ("name".to_string(), db::val::string("John Connor")),
//             ("age".to_string(), db::val::number(12)),
//         ],
//     );

//     let mut obj_storage = ObjectStorage::new();
//     let res = obj_storage.store_object(&datastore, object);
//     assert!(res.is_ok())
// }
