extern crate tree_sitter_cpp;
extern crate tree_sitter_viewer;

#[tokio::main]
async fn main() -> Result<(), rocket::Error> {

    // initial code to display
    let code = r#"void main() {
    printf("test");
}"#;

    // specify the parser's language and the initial code.
    let result = tree_sitter_viewer::run(tree_sitter_cpp::language(), code );

    result.await
}
