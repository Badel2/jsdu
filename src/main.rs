use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <path-to-file>", args[0]);
        std::process::exit(1);
    }

    let path = &args[1];

    let json = match fs::read_to_string(path) {
        Ok(contents) => {
            contents
        },
        Err(e) => {
            eprintln!("Error reading from {}: {}", path, e);
            std::process::exit(1);
        },
    };

    let mut js = &mut jsdu::size::JsonSize::new(&json);
    if let Some(json_path) = args.get(2) {
        js = js.index_json_path(json_path);
    }
    for l in js.display_list(&json) {
        println!("{}", l);
    }
}
