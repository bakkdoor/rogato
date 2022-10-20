use rogato_common::ast::helpers::{fn_def, op_call, var};
use rogato_compiler::Compiler;
#[allow(unused_imports)]
use rogato_interpreter::{EvalContext, Evaluate};
use rogato_parser::{parse, ParserContext};

use clap::Parser;
use indent_write::indentable::Indentable;
use std::fmt::{Debug, Display};
use std::fs::File;
use std::io::Read;
use std::path::Path;

mod repl;

// const DB_PATH: &str = "./rogato.db";

#[derive(Parser, Debug)]
#[clap(author,version,about,long_about=None)]
struct CLIArgs {
    #[clap(short, long, value_parser)]
    name: String,
}

fn main() -> anyhow::Result<()> {
    let args = std::env::args().skip(1);
    if args.len() == 0 {
        println!("No arguments given, but required.");
        print_help();
        return Ok(());
    }
    let mut help_required = false;
    let parser_ctx = ParserContext::new();
    for arg in args {
        match arg.as_str() {
            "help" => help_required = true,
            "repl" => {
                repl::run_repl()?;
            }
            "db" => {
                // println!("Running db tests");
                // println!("Opening DB @ {}", DB_PATH);
                // let datastore = db::open(Path::new(DB_PATH)).map_err(print_error).unwrap();
                // db_stuff(&datastore).unwrap();
            }
            "compile" => {
                let context = Compiler::new_context();
                let mut compiler = Compiler::new_with_module_name(&context, "compile_test");
                let func_name = "add3";
                let func_def = fn_def(
                    func_name,
                    ["x", "y", "z"],
                    op_call("*", op_call("+", var("x"), var("y")), var("z")),
                );

                compiler.compile_ast(func_def.as_ref())?;

                unsafe {
                    let function = compiler
                        .execution_engine()
                        .get_function::<unsafe extern "C" fn(f32, f32, f32) -> f32>(func_name)
                        .unwrap();

                    let params_and_results = [
                        ((1.1, 2.22, 3.333), 11.06556),
                        ((1.0, 2.0, 3.0), 9.0),
                        ((0.0, 0.0, 0.0), 0.0),
                        ((1.0, 0.0, 0.0), 0.0),
                        ((0.0, 2.2, 0.0), 0.0),
                        ((0.0, 0.0, 3.3), 0.0),
                        ((0.5, 10.0, 2.5), 26.25),
                    ];

                    for ((x, y, z), result) in params_and_results {
                        let val = function.call(x, y, z);
                        assert_eq!(val, result);
                        println!("{}({}, {}, {}) = {}", func_name, x, y, z, val);
                    }
                }
            }
            file => {
                println!("Attempting file parse: {}", file);
                let file_path = Path::new(file);
                if file_path.exists() {
                    read_parse_file(file_path, &parser_ctx);
                } else {
                    eprintln!("File not found: {:?}. Aborting.", file);
                    help_required = true;
                    break;
                }
            }
        }
    }

    if help_required {
        print_help()
    }

    Ok(())
}

fn read_parse_file(file_path: &Path, parser_ctx: &ParserContext) {
    match File::open(file_path) {
        Ok(mut file) => {
            let mut buf = String::new();
            file.read_to_string(&mut buf).unwrap();
            println!("\n📂\t{}", file_path.display());
            let parse_result = parse(buf.as_str(), parser_ctx);
            print_parse_result(buf.as_str(), &parse_result);
        }
        Err(error) => {
            println!("Could not open source file: {:?}", error);
        }
    }
}

fn print_help() {
    println!("Possible arguments:");
    println!("  help\n  repl\n  db\n  <source file path>");
}

fn print_parse_result<T: Display, E: Display>(code: &str, result: &Result<T, E>) {
    let lines = code.split('\n');
    let line_count = Vec::from_iter(lines.to_owned()).len();
    let (_, code_with_line_numbers) = lines.fold((1, String::new()), |(counter, acc), line| {
        let mut string = format!("{}\n{:02}  {}", acc, counter, line);
        if line_count > 100 {
            string = format!("{}\n{:03}  {}", acc, counter, line)
        }
        if line_count > 1000 {
            string = format!("{}\n{:03}  {}", acc, counter, line)
        }

        (counter + 1, string)
    });

    match result {
        Ok(expr) => println!("🌳 ✅\n{}\n\n", expr.indented("\t")),
        Err(error) => println!("❌{}\n\n❌\t{}\n\n", code_with_line_numbers, error),
    }
}

// pub fn db_stuff<DB: db::Datastore + Debug>(db: &DB) -> db::DBResult<()> {
//     basic_db_example(db)?;
//     complex_db_example(db)
// }

// pub fn basic_db_example<DB: db::Datastore + Debug>(db: &DB) -> db::DBResult<()> {
//     println!("DB: do stuff with {:?}", db);
// let person_type_id = db::id("Person");
// let name_prop_id = db::id("name");
// let age_prop_id = db::id("name");
// let friendship_edge_id = db::id("FriendShip");
// let bff_tag_id = db::id("bff");
// let nickname_prop_id = db::id("nickname");

// db.index_property(name_prop_id.to_owned())?;
// db.index_property(age_prop_id.to_owned())?;
// db.index_property(bff_tag_id.clone())?;
// db.index_property(nickname_prop_id.clone())?;

// for i in 0..1000 {
//     let id1 = db.create_vertex_from_type(person_type_id.to_owned())?;
//     let id2 = db.create_vertex_from_type(person_type_id.to_owned())?;

//     let friendship_edge_key = db::EdgeKey::new(id1, friendship_edge_id.clone(), id2);

//     db.bulk_insert(vec![
//         db::BulkInsertItem::VertexProperty(
//             id1,
//             name_prop_id.clone(),
//             val::string(format!("John Connor {}", i)),
//         ),
//         db::BulkInsertItem::VertexProperty(
//             id2,
//             name_prop_id.clone(),
//             val::string(format!("John Connor {}", i)),
//         ),
//         db::BulkInsertItem::VertexProperty(id1, age_prop_id.clone(), val::number(i * 1000)),
//         db::BulkInsertItem::VertexProperty(id2, age_prop_id.clone(), val::number(i * 9999)),
//         db::BulkInsertItem::Edge(friendship_edge_key.clone()),
//         db::BulkInsertItem::EdgeProperty(
//             friendship_edge_key.clone(),
//             bff_tag_id.clone(),
//             val::bool(true),
//         ),
//         db::BulkInsertItem::VertexProperty(
//             id2,
//             nickname_prop_id.clone(),
//             val::string(format!("Johnny {}", i)),
//         ),
//     ])?;
// }

// let prop_val_vtx_q =
//     db::PropertyValueVertexQuery::new(nickname_prop_id.clone(), val::string("Johnny 101"));

// let vtx = db.get_vertices(db::VertexQuery::PropertyValue(prop_val_vtx_q.clone()))?;
// let vtx_props = db.get_vertex_properties(prop_val_vtx_q.property(nickname_prop_id))?;

// println!("vtx: {:?}", vtx);
// println!("vtx_props: {:?}", vtx_props);

// let vertices = db.get_vertices(
//     db::RangeVertexQuery::new()
//         .t(person_type_id.to_owned())
//         .into(),
// )?;
// println!(
//     "Vertex query results for type {} : {}",
//     person_type_id.as_str(),
//     vertices.len()
// );

// let vertices_with_name = db.get_vertices(db::VertexQuery::PropertyValue(
//     db::PropertyValueVertexQuery::new(name_prop_id.clone(), val::string("John Connor 1")),
// ))?;
// println!(
//     "vertices_with_name query results: {}",
//     vertices_with_name.len()
// );

// let vertex_props = db.get_vertex_properties(db::VertexPropertyQuery::new(
//     db::VertexQuery::PropertyPresence(db::PropertyPresenceVertexQuery::new(age_prop_id)),
//     name_prop_id.clone(),
// ))?;
// println!(
//     "vertex_props query (age presence) result count: {}",
//     vertex_props.len()
// );

// let edge_query =
//     db::EdgeQuery::PropertyPresence(db::PropertyPresenceEdgeQuery::new(bff_tag_id));

// let vertex_props = db.get_vertex_properties(db::VertexPropertyQuery::new(
//     db::VertexQuery::Pipe(db::PipeVertexQuery::new(
//         Box::new(edge_query),
//         db::EdgeDirection::Inbound,
//     )),
//     name_prop_id,
// ))?;
// println!(
//     "vertex_props query (age presence, incoming Friendship edge) result count: {}",
//     vertex_props.len()
// );

//     Ok(())
// }

// pub fn complex_db_example<DB: db::Datastore + Debug>(db: &DB) -> db::DBResult<()> {
//     // example with some city and country data
//     let mut os = db::ObjectStorage::new();

//     // types
//     let t_city = os.get_id("City");
//     let t_country = os.get_id("Country");
//     let types = vec![t_city, t_country];

//     // properties
//     let meta_author_p = os.get_id("meta-author");
//     let properties = vec![meta_author_p.clone()];

//     println!("types:\n{:?}\nproperties:\n{:?}", types, properties);

//     db.index_property(meta_author_p)?;

//     // add query on meta_author vertex property (json object)
//     // with the following properties:
//     // meta-author:
//     //  - name
//     //  - created_at timestamp
//     //  - updated_at timestamp
//     //  - updated_at timestamp

//     Ok(())
// }
