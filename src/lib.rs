#[macro_use]
extern crate rocket;

use rocket::http::ContentType;
use rocket::http::Status;
use rocket::response::{content, status};
use std::path::{Path, PathBuf};
use std::{thread, time};

use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rust_embed::RustEmbed;
use serde_json::json;
use serde_json::{Map, Value};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use rocket::State;
use std::sync::Mutex;

#[derive(RustEmbed)]
#[folder = "./frontend/dist/"]
struct Asset;

#[get("/<file..>")]
fn index(file: PathBuf) -> (Status, (ContentType, Vec<u8>)) {
    println!("{:?}", file.as_path());

    let path = file.as_path();

    if path.to_str() == Some("") {
        let index_html = Asset::get("index.html").unwrap();
        (
            Status::Ok,
            (ContentType::HTML, index_html.data.as_ref().to_vec()),
        )
    } else {
        let file_content = Asset::get(path.to_str().unwrap());
        match file_content {
            Some(content) => {
                let extension = path.extension().unwrap().to_str();
                match extension {
                    None => {
                        println!("{} has no extension", path.to_str().unwrap());
                        (
                            Status::NotFound,
                            (ContentType::Binary, content.data.as_ref().to_vec()),
                        )
                    }
                    Some(extension) => {
                        let content_type = ContentType::from_extension(extension);
                        match content_type {
                            Some(content_type) => {
                                (Status::Ok, (content_type, content.data.as_ref().to_vec()))
                            }
                            None => match extension {
                                "map" => (
                                    Status::Ok,
                                    (
                                        ContentType::new("application", "json"),
                                        content.data.as_ref().to_vec(),
                                    ),
                                ),
                                _ => {
                                    println!("unknown extension {}", extension);

                                    (
                                        Status::Ok,
                                        (ContentType::Binary, content.data.as_ref().to_vec()),
                                    )
                                }
                            },
                        }
                    }
                }
            }
            None => (
                Status::NotFound,
                (ContentType::HTML, "<p>Not Found!</p>".as_bytes().to_vec()),
            ),
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(crate = "rocket::serde")]
struct CodePayload {
    code: String,
}

#[derive(Clone, Debug)]
pub struct Range {
    byte_start: usize,
    byte_end: usize,
}


#[derive(Clone, Debug)]
pub struct ASTNode {
    pub kind: String,
    pub children: Vec<Rc<RefCell<ASTNode>>>,
    pub range: Range,
    pub name: Option<String>,
    pub id: usize,
    pub content: Option<String>
}

impl ASTNode {
    pub fn add_child(&mut self, c: &Rc<RefCell<ASTNode>>) {
        self.children.push(c.clone());
    }
}

fn json_from_ast(ast: &Rc<RefCell<ASTNode>>) -> serde_json::Map<String, serde_json::Value> {
    let mut map: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();

    map.insert("kind".to_string(), json!(ast.borrow().kind));
    map.insert("label".to_string(), json!(ast.borrow().kind));
    map.insert("id".to_string(), json!(ast.borrow().id));

    if ast.borrow().content.is_some() {
        map.insert("content".to_string(), json!(ast.borrow().content.as_ref().unwrap()));
    }

    if ast.borrow().name.is_some() {
        map.insert("name".to_string(), json!(ast.borrow().name.as_ref().unwrap()));
    }

    let mut vec: Vec<serde_json::Value> = Vec::<serde_json::Value> ::new();

    if ast.borrow().children.len() > 0 {
        for c in &ast.borrow().children {
            vec.push(serde_json::Value::Object(json_from_ast(&c)));
        }
    }

    map.insert("children".to_string(), serde_json::Value::Array(vec));

    map
}

#[post("/api/update_code", format = "json", data = "<payload>")]
fn update_code(payload: Json<CodePayload>, global: &State<Arc<GlobalState>>) -> (Status, (ContentType, Vec<u8>)) {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(*global.language.lock().expect("lock language"))
        .expect("Error loading grammar");

    let parsed = parser.parse(&payload.code, None).unwrap();

    println!("{}", parsed.root_node().to_sexp());
    let bytes = payload.code.as_bytes();
    let mut cursor: tree_sitter::TreeCursor = parsed.root_node().walk();

    let curr: Rc<RefCell<ASTNode>> = Rc::new(RefCell::<ASTNode>::new(ASTNode {
        kind: String::new(),
        children: Vec::<Rc<RefCell<ASTNode>>>::new(),
        range: Range {
            byte_start: 0,
            byte_end: 0,
        },
        name: None,
        id: 0,
        content: None
    }));

    let mut stack: Vec::<Rc<RefCell<ASTNode>>> = Vec::<Rc<RefCell<ASTNode>>>::new();
    stack.push(curr.clone());

    let mut reached_root = false;
    while reached_root == false {

        let c = Rc::new(RefCell::<ASTNode>::new( 
            ASTNode {
                kind: cursor.node().kind().to_string(),
                children: Vec::<Rc<RefCell<ASTNode>>>::new(),
                range: Range {
                    byte_start: cursor.node().range().start_byte,
                    byte_end: cursor.node().range().end_byte,
                },
                name: if let Some(n) = cursor.field_name() {Some(n.to_string())} else {None},
                id: cursor.node().id(),
                content: None
            }
         ));

        stack.last_mut().unwrap().borrow_mut().children.push(c.clone());

        if cursor.goto_first_child() {
            stack.push(c);
            continue;
        }
        else {
            let slice = &bytes[c.borrow().range.byte_start..c.borrow().range.byte_end];
            let content = std::str::from_utf8(slice).unwrap();

            if content != cursor.node().kind() {
                c.borrow_mut().content = Some(content.to_string());
            }

            if cursor.goto_next_sibling() {
                
    
                continue;
            }
        }
        
        
        let mut retracing = true;
        while retracing {
            if !cursor.goto_parent() {
                retracing = false;
                reached_root = true;
            } else {
                stack.pop();
            }
            if cursor.goto_next_sibling() {
                retracing = false;
            }
        }
        if reached_root {
            break;
        }
    }

    let root: Rc<RefCell<ASTNode>> = curr.borrow().children[0].clone();

    println!("root c len {}", root.borrow().children.len());


    let map: serde_json::Map<String, serde_json::Value> = json_from_ast(&root);


    (
        Status::Ok,
        (ContentType::JSON, serde_json::to_vec(&map).unwrap()),
    )
}

struct GlobalState {
    language: Mutex<tree_sitter::Language>
}

pub async fn run(language: tree_sitter::Language) -> Result<(), rocket::Error> {
    println!("Tree-sitter viewer");

    let global = Arc::new(GlobalState{language: Mutex::new(language)});

    let rocket = rocket::build().mount("/", routes![index, update_code]).manage(global);

    let result = rocket.launch().await;

    println!("server shutdown");
    result
}
