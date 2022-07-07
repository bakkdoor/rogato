mod fmodl;
use std::fmt::{Debug, Display};
use std::io::{self, Read};
use std::path::Path;

use fmodl::db;
#[allow(unused_imports)]
use fmodl::db::{EdgeQueryExt, VertexQueryExt};
use fmodl::parser::{parse, parse_expr};
use indent_write::indentable::Indentable;
use std::fs::File;

use crate::fmodl::util::print_error;

const DB_PATH: &str = "./fmodl.db";

#[cfg(test)]
mod tests;

fn main() {
    let args = std::env::args().skip(1);
    if args.len() == 0 {
        println!("No arguments given, but required.");
        print_help();
        return;
    }
    let mut help_required = false;
    for arg in args {
        match arg.as_str() {
            "help" => help_required = true,
            "repl" => {
                println!("Running REPL");
                run_repl();
            }
            "parse" => {
                println!("Running parse tests");
                try_parse_root_defs();
                try_parse_expressions();
            }
            "examples" => {
                println!("Trying to parse example files");
                match std::fs::read_dir(Path::new("examples/")) {
                    Ok(rd) => {
                        for e in rd {
                            let dir_entry = e.unwrap();
                            match File::open(dir_entry.path()) {
                                Ok(mut file) => {
                                    let mut buf = String::new();
                                    file.read_to_string(&mut buf).unwrap();
                                    println!("\n📂\t{}", dir_entry.path().display());
                                    print_parse_result(buf.as_str(), parse(buf.as_str()))
                                }
                                Err(error) => {
                                    println!("Could not open example source file: {:?}", error)
                                }
                            }
                        }
                    }
                    Err(_) => {}
                }
            }
            "db" => {
                println!("Running db tests");
                println!("Opening DB @ {}", DB_PATH);
                let datastore = db::open(Path::new(DB_PATH)).map_err(print_error).unwrap();
                db_stuff(&datastore).unwrap();
            }
            _ => {
                println!("Unknown argument: {:?}", arg);
                help_required = true;
            }
        }
    }

    if help_required {
        print_help()
    }
}

fn print_help() {
    println!("Possible arguments:");
    println!("  help\n  repl\n  parse\n  examples\n  db");
}

fn try_parse_root_defs() {
    for root_def in [
        "module MyModule",
        "module MyModule ( foo, bar, baz )",
        "module MyModule (  foo,     bar,   baz   )",
        "let squared x = x",
        "let add a b = 1 + b",
        "let addTwice a b = 2 * (a + b)",
        "let complexMath a b c =
            let x = 2 * (a + b * c),
                y = y * a + b * c,
                z = addTwice (squared (x * y)) (x * y)
            in
                y * z
        ",
    ] {
        print_parse_result(root_def, parse(root_def))
    }
}

fn try_parse_expressions() {
    for expr_code in [
        "1",
        "1+1",
        "2+(3+4)",
        "1*2",
        "(2*3)*(3+5)",
        "2 * 3 + 4 * 8",
        "3 + (4 * 5)",
        "myFunction 2 3",
        "foo (bar 1)",
        "foo (bar 1) 2 3",
        "foo (bar (baz (3 * ((4 * 5) * (6 + 7)))))",
        "to-upper a b",
        "      to-upper   (  __do-something-with__         a-var        b_var      )   ",
    ] {
        print_parse_result(expr_code, parse_expr(expr_code))
    }
}

fn print_parse_result<T: Display, E: Display>(code: &str, result: Result<T, E>) {
    let lines = code.split("\n");
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
        Ok(expr) => println!(
            "✅\t{}\n\n🧾 ✅\n{}\n\n",
            code_with_line_numbers,
            expr.indented("\t")
        ),
        Err(error) => println!("\n❌\t{}\n\n❌\t{}\n\n", code_with_line_numbers, error),
    }
}

fn run_repl() {
    loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        match parse(buffer.as_str()) {
            Ok(exp) => {
                println!("OK> {:?}\n{}", exp, exp);
            }
            Err(err) => {
                eprintln!("Error> {:?}", err)
            }
        }
    }
}

pub fn db_stuff<DB: db::Datastore + Debug>(db: &DB) -> db::DBResult<()> {
    // basic_db_example(db)?;
    complex_db_example(db)
}

pub fn basic_db_example<DB: db::Datastore + Debug>(db: &DB) -> db::DBResult<()> {
    println!("DB: do stuff with {:?}", db);
    let person_type_id = db::id("Person");
    let name_prop_id = db::id("name");
    let age_prop_id = db::id("name");
    let friendship_edge_id = db::id("FriendShip");
    let bff_tag_id = db::id("bff");
    let nickname_prop_id = db::id("nickname");

    db.index_property(name_prop_id.to_owned())?;
    db.index_property(age_prop_id.to_owned())?;
    db.index_property(bff_tag_id.clone())?;
    db.index_property(nickname_prop_id.clone())?;

    for i in 0..1000 {
        let id1 = db.create_vertex_from_type(person_type_id.to_owned())?;
        let id2 = db.create_vertex_from_type(person_type_id.to_owned())?;

        let friendship_edge_key =
            db::EdgeKey::new(id1.clone(), friendship_edge_id.clone(), id2.clone());

        db.bulk_insert(vec![
            db::BulkInsertItem::VertexProperty(
                id1.clone(),
                name_prop_id.clone(),
                db::val::string(format!("John Connor {}", i)),
            ),
            db::BulkInsertItem::VertexProperty(
                id2.clone(),
                name_prop_id.clone(),
                db::val::string(format!("John Connor {}", i)),
            ),
            db::BulkInsertItem::VertexProperty(
                id1.clone(),
                age_prop_id.clone(),
                db::val::number(i * 1000),
            ),
            db::BulkInsertItem::VertexProperty(
                id2.clone(),
                age_prop_id.clone(),
                db::val::number(i * 9999),
            ),
            db::BulkInsertItem::Edge(friendship_edge_key.clone()),
            db::BulkInsertItem::EdgeProperty(
                friendship_edge_key.clone(),
                bff_tag_id.clone(),
                db::val::bool(true),
            ),
            db::BulkInsertItem::VertexProperty(
                id2.clone(),
                nickname_prop_id.clone(),
                db::val::string(format!("Johnny {}", i)),
            ),
        ])?;
    }

    let prop_val_vtx_q =
        db::PropertyValueVertexQuery::new(nickname_prop_id.clone(), db::val::string("Johnny 101"));

    let vtx = db.get_vertices(db::VertexQuery::PropertyValue(prop_val_vtx_q.clone()))?;
    let vtx_props =
        db.get_vertex_properties(prop_val_vtx_q.clone().property(nickname_prop_id.clone()))?;

    println!("vtx: {:?}", vtx);
    println!("vtx_props: {:?}", vtx_props);

    let vertices = db.get_vertices(
        db::RangeVertexQuery::new()
            .t(person_type_id.to_owned())
            .into(),
    )?;
    println!(
        "Vertex query results for type {} : {}",
        person_type_id.to_owned().as_str(),
        vertices.len()
    );

    let vertices_with_name = db.get_vertices(indradb::VertexQuery::PropertyValue(
        db::PropertyValueVertexQuery::new(name_prop_id.clone(), db::val::string("John Connor 1")),
    ))?;
    println!(
        "vertices_with_name query results: {}",
        vertices_with_name.len()
    );

    let vertex_props = db.get_vertex_properties(db::VertexPropertyQuery::new(
        db::VertexQuery::PropertyPresence(db::PropertyPresenceVertexQuery::new(
            age_prop_id.clone(),
        )),
        name_prop_id.clone(),
    ))?;
    println!(
        "vertex_props query (age presence) result count: {}",
        vertex_props.len()
    );

    let edge_query =
        db::EdgeQuery::PropertyPresence(db::PropertyPresenceEdgeQuery::new(bff_tag_id.clone()));

    let vertex_props = db.get_vertex_properties(db::VertexPropertyQuery::new(
        db::VertexQuery::Pipe(db::PipeVertexQuery::new(
            Box::new(edge_query),
            db::EdgeDirection::Inbound,
        )),
        name_prop_id.clone(),
    ))?;
    println!(
        "vertex_props query (age presence, incoming Friendship edge) result count: {}",
        vertex_props.len()
    );

    Ok(())
}

pub fn complex_db_example<DB: db::Datastore + Debug>(db: &DB) -> db::DBResult<()> {
    // example with some city and country data

    // types
    let t_city = db::id("City");
    let t_country = db::id("Country");
    let types = vec![t_city, t_country];

    // properties
    let p_meta_author = db::id("meta-author");
    let properties = vec![p_meta_author.clone()];

    println!("types:\n{:?}\nproperties:\n{:?}", types, properties);

    db.index_property(p_meta_author.clone())?;

    // add query on meta_author vertex property (json object)
    // with the following properties:
    // meta-author:
    //  - name
    //  - created_at timestamp
    //  - updated_at timestamp
    //  - updated_at timestamp

    Ok(())
}
