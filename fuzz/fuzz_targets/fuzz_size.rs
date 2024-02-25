#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data_str: &str| {
    // fuzzed code goes here
    if serde_json::from_str::<serde::de::IgnoredAny>(data_str).is_err() {
        return;
    }

    //println!("{}", data_str);
    for _l in jsdu::size::JsonSize::new(data_str).display_list(data_str) {
        //println!("{}", l);
    }

});
