mod minify;
mod prettify;
mod size;

fn main() {
    let json = r#"[-234.67e9, 0]"#;
    let json = r#"{"s": "19 character string", "n": -234.67e9, "boolean": true}"#;
    for l in size::JsonSize::new(json).display_list(json) {
        println!("{}", l);
    }
}