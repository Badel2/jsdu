#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data_str: &str| {
    // fuzzed code goes here
    let x1 = serde_json::from_str::<serde_json::Value>(data_str);
    if x1.is_err() {
        return;
    }
    let x1 = x1.unwrap();

    //println!("{}", data_str);
    let minified_string = jsdu::minify::minify(data_str);
    if minified_string == data_str {
        return;
    }
    let x2 = serde_json::from_str::<serde_json::Value>(&minified_string);
    assert_eq!(x1, x2.unwrap());

    // Check that we minify it the same way as serde_json
    // Disabled because it is not true :(
    // 0 => 0.0
    // "\/" => "/"
    /*
    if !has_any_numbers_inside(&x1) {
        let minified_by_serde_json = serde_json::to_string(&x1).unwrap();
        assert_eq!(minified_string, minified_by_serde_json);
    }
    */
});

fn has_any_numbers_inside(x: &serde_json::Value) -> bool {
    match x {
        serde_json::Value::Number(_) => true,
        serde_json::Value::Null => false,
        serde_json::Value::Bool(_) => false,
        serde_json::Value::String(_) => false,
        _ => true,
    }
}
