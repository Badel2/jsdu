pub fn minify(json: &str) -> String {
    MinifyIter::new(json).collect()
}

pub struct MinifyIter<I> {
    chars: I,
    in_string: bool,
    escape_next: bool,
}

impl<'a> MinifyIter<std::str::Chars<'a>> {
    pub fn new(json: &'a str) -> Self {
        MinifyIter {
            chars: json.chars(),
            in_string: false,
            escape_next: false,
        }
    }
}

impl<I: Iterator<Item = char>> Iterator for MinifyIter<I> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(c) = self.chars.next() {
            if self.escape_next {
                self.escape_next = false;
                // Return the escaped character
                return Some(c);
            }

            match c {
                '"' => {
                    self.in_string = !self.in_string;
                    return Some(c);
                }
                '\\' if self.in_string => {
                    // Escape next character (so \" inside a string is not treated as end of string)
                    self.escape_next = true;
                    // Return the escape character
                    return Some(c);
                }
                // Skip whitespace outside of strings
                c if c.is_whitespace() && !self.in_string => continue,
                _ => return Some(c),
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_minifies_json() {
        let json = r#"{
            "key": "value",
            "array": [1, 2, 3]
        }"#;

        let minified = minify(json);

        assert_eq!(minified, r#"{"key":"value","array":[1,2,3]}"#);
    }

    #[test]
    fn it_handles_escaped_characters_in_string() {
        let json = r#"{"escaped": "Contains \\\"escaped\\\" characters and \"whitespace inside quotes\""}"#;

        let minified = minify(json);

        assert_eq!(
            minified,
            r#"{"escaped":"Contains \\\"escaped\\\" characters and \"whitespace inside quotes\""}"#
        );
    }
}
