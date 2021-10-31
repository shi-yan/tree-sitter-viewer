extern crate tree_sitter_cpp;
extern crate tree_sitter_viewer;

#[tokio::main]
async fn main() -> Result<(), rocket::Error> {

    let result = tree_sitter_viewer::run(tree_sitter_cpp::language());

    result.await
}
