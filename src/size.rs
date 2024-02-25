//! Given a json file, calculate the size of each item.
//! Display the data in a format similar to ncdu.
use std::iter::Peekable;

#[derive(Default, Debug, PartialEq, Eq)]
pub struct JsonSize {
    /// Size that will disappear after minifying the json file
    whitespace: usize,
    /// Commas, quotes, and other JSON characters
    control_chars: usize,
    /// Actual data: strings, numbers, and keywords
    data_size: usize,
    data_ptr: Span,
    children: Vec<JsonSize>,
    value_kind: JsonValueKind,
    key: JsonKey,
}

impl JsonSize {
    pub fn new(json: &str) -> Self {
        // Assumes valid json
        let mut chars = json.char_indices().peekable();
        let js = parse_json_size(&mut chars, 0).0;

        // Invariant: whitespace + control_chars + data_size == input.len()
        assert_total_size_invariant(json, &js);

        js
    }

    fn add_stats_from(&mut self, other: &JsonSize) {
        self.whitespace += other.whitespace;
        self.control_chars += other.control_chars;
        self.data_size += other.data_size;
    }

    fn total_size(&self) -> usize {
        self.whitespace + self.control_chars + self.data_size
    }

    pub fn display_list(&self, json: &str) -> Vec<String> {
        if self.children.is_empty() {
            let percent = "[##########]";
            //let name = self.key.to_display(json);
            let name = "Total";
            let line = format!("{:12} {} {}", self.total_size(), percent, name);
            vec![line]
        } else {
            let percent = "[##########]";
            //let name = self.key.to_display(json);
            let name = "Total";
            let total_size = self.total_size();
            let line = format!("{:12} {} {}", total_size, percent, name);
            let mut lines = vec![line];
            for child in self.children.iter() {
                let mut percent = b"          ".to_vec();
                let name = child.key.to_display(json);
                let size = child.total_size();
                for i in 0..10 {
                    if size > total_size * i / percent.len() {
                        percent[i] = b'#';
                    }
                }
                let line = format!("{:12} [{}] {}", size, String::from_utf8(percent).unwrap(), name);
                lines.push(line);
            }
            lines
        }
    }
}

#[derive(Default, Debug, PartialEq, Eq)]
struct JsonKey {
    index: usize,
    key_ptr: Option<Span>,
}

impl JsonKey {
    fn to_display(&self, json: &str) -> String {
        if let Some(key_ptr) = &self.key_ptr {
            let key_ptr_end = find_string_end(*key_ptr, json);
            format!("\"{}\"", &json[key_ptr.start..key_ptr_end])
        } else {
            format!("{}", self.index)
        }
    }
}

fn find_string_end(key_ptr: Span, json: &str) -> usize {
    let mut key_ptr = key_ptr.start;
    let b = json.as_bytes();
    let mut escape_next = false;

    loop {
        let c = b[key_ptr];
        if escape_next {
            escape_next = false;
            key_ptr += 1;
            continue;
        }
        if c == b'"' {
            return key_ptr;
        }
        if c == b'\\' {
            escape_next = true;
        }
        key_ptr += 1;
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
struct Span {
    start: usize,
}

#[derive(Default, Debug, PartialEq, Eq)]
enum JsonValueKind {
    #[default]
    Null,
    Boolean,
    Number,
    String,
    Object,
    Array,
}

fn parse_json_size<I>(mut chars: &mut Peekable<I>, recursion_level: usize) -> (JsonSize, Option<char>, bool)
where I: Iterator<Item = (usize, char)>
{
    let mut js = JsonSize {
        whitespace: 0,
        control_chars: 0,
        children: vec![],
        value_kind: JsonValueKind::Null,
        data_size: 0,
        data_ptr: Span::default(),
        key: JsonKey::default(),
    };
    let mut is_empty = true;

    loop {
        let c = chars.peek();
        println!("parse_json_size: {:?}", c);
        if c.is_none() {
            break;
        }
        let (c_ptr, c) = *c.unwrap();
        match c {
            ' ' | '\n' | '\r' | '\t' => {
                js.whitespace += 1;
                chars.next().unwrap();
            }
            't' | 'f' | 'n' => {
                js.value_kind = if c == 'n' { JsonValueKind::Null } else { JsonValueKind::Boolean };
                is_empty = false;
                parse_any_keyword(&mut chars, &mut js).unwrap();
            }
            '"' => {
                js.value_kind = JsonValueKind::String;
                is_empty = false;
                parse_string(&mut chars, &mut js).unwrap();
            }
            '-' | '0'..='9' => {
                js.value_kind = JsonValueKind::Number;
                is_empty = false;
                parse_number(&mut chars, &mut js).unwrap();
            }
            '[' => {
                js.value_kind = JsonValueKind::Array;
                is_empty = false;
                parse_array(&mut chars, &mut js, recursion_level + 1).unwrap();
            }
            '{' => {
                js.value_kind = JsonValueKind::Object;
                is_empty = false;
                parse_object(&mut chars, &mut js, recursion_level + 1).unwrap();
            }
            ',' | ']' | '}' => {
                if recursion_level == 0 {
                    panic!("Invalid json");
                } else {
                    return (js, Some(c), is_empty);
                }
            }
            _ => todo!(),
        }
    }

    if recursion_level == 0 {
        (js, None, is_empty)
    } else {
        panic!("Invalid json");
    }
}

fn parse_array<I>(mut chars: &mut Peekable<I>, js: &mut JsonSize, recursion_level: usize) -> Result<(), ()>
where I: Iterator<Item = (usize, char)>
{
    // Save start index
    let data_ptr = chars.peek().ok_or(())?.0;
    let mut len = 0;
    // Remove leading [
    match chars.next().ok_or(())?.1 {
        '[' => (),
        _ => return Err(()),
    }

    loop {
        let (mut child, last_char, is_empty) = parse_json_size(chars, recursion_level);
        child.key = JsonKey { index: js.children.len(), key_ptr: None };
        js.add_stats_from(&child);
        if !is_empty {
            js.children.push(child);
        }
        
        if last_char == Some(',') {
            // Remove comma
            match chars.next().ok_or(())?.1 {
                ',' => (),
                _ => return Err(()),
            }
            continue;
        } else {
            break;
        }
    }

    // Remove final ]
    match chars.next().ok_or(())?.1 {
        ']' => (),
        _ => return Err(()),
    }

    // Open and close array, and one comma per item except one
    js.control_chars += 2 + js.children.len().saturating_sub(1);

    Ok(())
}

fn parse_object<I>(mut chars: &mut Peekable<I>, js: &mut JsonSize, recursion_level: usize) -> Result<(), ()>
where I: Iterator<Item = (usize, char)>
{
    // Save start index
    let data_ptr = chars.peek().ok_or(())?.0;
    let mut len = 0;
    // Remove leading {
    match chars.next().ok_or(())?.1 {
        '{' => (),
        _ => return Err(()),
    }

    loop {
        let mut key_js = JsonSize::default();
        // Optional whitespace
        skip_whitespace(chars, &mut key_js);
        let key_ptr = chars.peek().ok_or(())?.0;
        // Remove "key"
        parse_string(chars, &mut key_js)?;
        // Optional whitespace
        skip_whitespace(chars, &mut key_js);
        // Remove :
        match chars.next().ok_or(())?.1 {
            ':' => (),
            _ => return Err(()),
        }
        js.add_stats_from(&key_js);

        // Remove value
        let (mut child, last_char, is_empty) = parse_json_size(chars, recursion_level);
        child.key = JsonKey { index: js.children.len(), key_ptr: Some(Span { start: key_ptr+1 }) };
        js.add_stats_from(&child);
        if !is_empty {
            js.children.push(child);
        }
        
        if last_char == Some(',') {
            // Remove comma
            match chars.next().ok_or(())?.1 {
                ',' => (),
                _ => return Err(()),
            }
            continue;
        } else {
            break;
        }
    }

    // Remove final }
    match chars.next().ok_or(())?.1 {
        '}' => (),
        _ => return Err(()),
    }

    // Open and close object, one colon per item, and one comma per item except one
    js.control_chars += 2 + js.children.len() + js.children.len().saturating_sub(1);

    Ok(())
}

fn parse_number<I>(mut chars: &mut Peekable<I>, js: &mut JsonSize) -> Result<(), ()>
where I: Iterator<Item = (usize, char)>
{
    // Save start index
    let data_ptr = chars.peek().ok_or(())?.0;
    let mut len = 0;
    // Remove optional leading minus sign
    match chars.peek().ok_or(())?.1 {
        '-' => {
            let c = chars.next().unwrap().1;
            len += c.len_utf8();
        }
        _ => (),
    }

    // Remove integer part
    // Allow leading 0, we assume the json is valid at this point
    loop {
        match chars.peek().map(|uc| uc.1) {
            Some('0'..='9') => {
                let c = chars.next().unwrap().1;
                len += c.len_utf8();
            }
            None => {
                js.data_size += len;
                js.control_chars += 0;
                js.data_ptr = Span { start: data_ptr };
                return Ok(());
            }
            _ => break,
        }
    }

    // Remove optional fraction part
    match chars.peek().map(|uc| uc.1) {
        Some('.') => {
            let c = chars.next().unwrap().1;
            len += c.len_utf8();

            loop {
                match chars.peek().map(|uc| uc.1) {
                    Some('0'..='9') => {
                        let c = chars.next().unwrap().1;
                        len += c.len_utf8();
                    }
                    None => {
                        js.data_size += len;
                        js.control_chars += 0;
                        js.data_ptr = Span { start: data_ptr };
                        return Ok(());
                    }
                    _ => break,
                }
            }
        }
        _ => (),
    }

    // Remove optional exponent part
    match chars.peek().map(|uc| uc.1) {
        Some('e' | 'E') => {
            let c = chars.next().unwrap().1;
            len += c.len_utf8();

            // Remove optional sign
            match chars.peek().ok_or(())?.1 {
                '-' | '+' => {
                    let c = chars.next().unwrap().1;
                    len += c.len_utf8();
                }
                _ => (),
            }

            // Remove exponent digits
            loop {
                match chars.peek().map(|uc| uc.1) {
                    Some('0'..='9') => {
                        let c = chars.next().unwrap().1;
                        len += c.len_utf8();
                    }
                    None => {
                        js.data_size += len;
                        js.control_chars += 0;
                        js.data_ptr = Span { start: data_ptr };
                        return Ok(());
                    }
                    _ => break,
                }
            }
        }
        _ => (),
    }

    js.data_size += len;
    js.control_chars += 0;
    js.data_ptr = Span { start: data_ptr };

    Ok(())
}

fn parse_string<I>(mut chars: &mut Peekable<I>, js: &mut JsonSize) -> Result<(), ()>
where I: Iterator<Item = (usize, char)>
{
    match chars.next().ok_or(())?.1 {
        '"' => (),
        _ => return Err(()),
    }

    let mut escape_next = false;
    let mut len = 0;

    loop {
        let c = chars.next();
        if c.is_none() {
            return Err(());
        }
        let c = c.unwrap().1;
        len += c.len_utf8();
        if escape_next {
            len += 1;
            escape_next = false;
            continue;
        }
        if c == '\\' {
            escape_next = true;
            continue;
        }
        if c == '"' {
            len -= 1;
            js.data_size += len;
            js.control_chars += 2;
            return Ok(());
        }
    }

    Ok(())
}

fn parse_any_keyword<I>(mut chars: &mut Peekable<I>, js: &mut JsonSize) -> Result<(), ()>
where I: Iterator<Item = (usize, char)>
{
    match chars.peek().ok_or(())?.1 {
        't' => parse_keyword("true", chars, js),
        'f' => parse_keyword("false", chars, js),
        'n' => parse_keyword("null", chars, js),
        _ => Err(())
    }
}

fn parse_keyword<I>(keyword: &'static str, mut chars: I, js: &mut JsonSize) -> Result<(), ()>
where I: Iterator<Item = (usize, char)>
{
    let keyword_len = keyword.len();
    let mut keyword_chars = keyword.chars();

    // Most ineficient memchr ever
    loop {
        let next_k = keyword_chars.next();
        if next_k.is_none() {
            js.data_size += keyword_len;
            return Ok(());
        }
        let next_i = chars.next().map(|uc| uc.1);

        if next_i != next_k {
            return Err(());
        }
    }
}

fn skip_whitespace<I>(mut chars: &mut Peekable<I>, js: &mut JsonSize)
where I: Iterator<Item = (usize, char)>
{
    loop {
        let c = chars.peek();
        if c.is_none() {
            return;
        }
        let c = c.unwrap().1;
        match c {
            ' ' | '\n' | '\r' | '\t' => {
                js.whitespace += 1;
                chars.next().unwrap();
            }
            _ => return
        }
    }
}

fn assert_total_size_invariant(json: &str, js: &JsonSize) {
    // Invariant: whitespace + control_chars + data_size == input.len()
    assert_eq!(json.len(), js.whitespace + js.control_chars + js.data_size, "{:?}\n{:?}", json, js);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_string_size() {
        let json = r#""19 character string""#;
        let js = JsonSize::new(json);
        assert_eq!(js.control_chars, 2);
        // Only counts whitespace outside of string
        assert_eq!(js.whitespace, 0);
        assert_eq!(js.data_size, 19);
        assert_eq!(js.value_kind, JsonValueKind::String);
        assert_eq!(js.children, vec![]);
    }

    #[test]
    fn test_empty_string_size() {
        let json = r#""""#;
        let js = JsonSize::new(json);
        assert_eq!(js.control_chars, 2);
        assert_eq!(js.whitespace, 0);
        assert_eq!(js.data_size, 0);
        assert_eq!(js.value_kind, JsonValueKind::String);
        assert_eq!(js.children, vec![]);
    }

    #[test]
    fn test_bool_size() {
        let json = r#"false"#;
        let js = JsonSize::new(json);
        assert_eq!(js.control_chars, 0);
        assert_eq!(js.whitespace, 0);
        assert_eq!(js.data_size, 5);
        assert_eq!(js.value_kind, JsonValueKind::Boolean);
        assert_eq!(js.children, vec![]);
    }

    fn assert_number_size(json: &str, size: usize) {
        let js = JsonSize::new(json);
        assert_eq!(js.control_chars, 0);
        assert_eq!(js.whitespace, 0);
        assert_eq!(js.data_size, size);
        assert_eq!(js.value_kind, JsonValueKind::Number);
        assert_eq!(js.children, vec![]);
    }

    #[test]
    fn test_number_size() {
        assert_number_size("1", 1);
        assert_number_size("12", 2);
        assert_number_size("123", 3);
        assert_number_size("1234", 4);
        assert_number_size("1234.6", 6);
        assert_number_size("1234.67", 7);
        assert_number_size("1234.67e9", 9);
        assert_number_size("1234.67E9", 9);
        assert_number_size("1234567E9", 9);
        assert_number_size("1234567e9", 9);
        assert_number_size("1234567e+11", 11);
        assert_number_size("1234567e-11", 11);
        assert_number_size("-2", 2);
        assert_number_size("-234.67e9", 9);
    }

    #[test]
    fn test_array() {
        let json = r#"["19 character string"]"#;
        let js = JsonSize::new(json);
        assert_eq!(js.control_chars, 4);
        // Only counts whitespace outside of string
        assert_eq!(js.whitespace, 0);
        assert_eq!(js.data_size, 19);
        assert_eq!(js.value_kind, JsonValueKind::Array);
        assert_eq!(js.children.len(), 1);
    }

    #[test]
    fn test_array_3() {
        let json = r#"["19 character string", -234.67e9, null]"#;
        let js = JsonSize::new(json);
        assert_eq!(js.control_chars, 6);
        // Only counts whitespace outside of string
        assert_eq!(js.whitespace, 2);
        assert_eq!(js.data_size, 19 + 9 + 4);
        assert_eq!(js.value_kind, JsonValueKind::Array);
        assert_eq!(js.children.len(), 3);
    }

    #[test]
    fn test_array_num() {
        let json = r#"[-234.67e9, 0]"#;
        let js = JsonSize::new(json);
        assert_eq!(js.control_chars, 3);
        // Only counts whitespace outside of string
        assert_eq!(js.whitespace, 1);
        assert_eq!(js.data_size, 9 + 1);
        assert_eq!(js.value_kind, JsonValueKind::Array);
        assert_eq!(js.children.len(), 2);
    }

    #[test]
    fn test_empty_array() {
        let json = r#"[]"#;
        let js = JsonSize::new(json);
        assert_eq!(js.control_chars, 2);
        // Only counts whitespace outside of string
        assert_eq!(js.whitespace, 0);
        assert_eq!(js.data_size, 0);
        assert_eq!(js.value_kind, JsonValueKind::Array);
        assert_eq!(js.children.len(), 0);
    }

    #[test]
    fn test_empty_whitespace() {
        let json = r#"  [   ]    "#;
        let js = JsonSize::new(json);
        assert_eq!(js.control_chars, 2);
        // Only counts whitespace outside of string
        assert_eq!(js.whitespace, 2+3+4);
        assert_eq!(js.data_size, 0);
        assert_eq!(js.value_kind, JsonValueKind::Array);
        assert_eq!(js.children.len(), 0);
    }

    #[test]
    fn test_object() {
        let json = r#"{"s": "19 character string"}"#;
        let js = JsonSize::new(json);
        assert_eq!(js.control_chars, 2+2+2+1);
        // Only counts whitespace outside of string
        assert_eq!(js.whitespace, 1);
        assert_eq!(js.data_size, 19+1);
        assert_eq!(js.value_kind, JsonValueKind::Object);
        assert_eq!(js.children.len(), 1);
    }

    #[test]
    fn test_object_3() {
        let json = r#"{"s": "19 character string", "n": -234.67e9, "boolean": true}"#;
        let js = JsonSize::new(json);
        assert_eq!(js.control_chars, 2+3+3+3+1+3);
        // Only counts whitespace outside of string
        assert_eq!(js.whitespace, 5);
        assert_eq!(js.data_size, 19+9+4 + 1+1+7);
        assert_eq!(js.value_kind, JsonValueKind::Object);
        assert_eq!(js.children.len(), 3);
    }
}
