use std::collections::VecDeque;

pub fn prettify(json: &str, indent_width: usize) -> String {
    PrettifyIter::new(json, indent_width).collect()
}

pub struct PrettifyIter<'a> {
    chars: std::str::Chars<'a>,
    buffer_0: Option<BufferChar>,
    buffer_1: Option<BufferChar>,
    space_buffer: usize,
    indent_width: usize,
    current_indent: usize,
    in_string: bool,
    escape_next: bool,
}

enum BufferChar {
    Space,
    ManySpaces,
    NewLine,
    EndObject,
    EndArray,
}

impl From<BufferChar> for char {
    fn from(x: BufferChar) -> Self {
        match x {
            BufferChar::Space => ' ',
            BufferChar::ManySpaces => panic!("Handle `ManySpaces` before converting to char"),
            BufferChar::NewLine => '\n',
            BufferChar::EndObject => '}',
            BufferChar::EndArray => ']',
        }
    }
}

impl<'a> PrettifyIter<'a> {
    pub fn new(json: &'a str, indent_width: usize) -> Self {
        Self {
            chars: json.chars(),
            buffer_0: None,
            buffer_1: None,
            space_buffer: 0,
            indent_width,
            current_indent: 0,
            in_string: false,
            escape_next: false,
        }
    }

    fn buffer_push_back(&mut self, x: BufferChar) {
        if self.buffer_0.is_none() {
            self.buffer_0 = Some(x);
        } else {
            assert!(self.buffer_1.is_none());
            self.buffer_1 = Some(x);
        }
    }

    fn buffer_push_front(&mut self, x: BufferChar) {
        if self.buffer_0.is_none() {
            self.buffer_0 = Some(x);
        } else {
            assert!(self.buffer_1.is_none());
            self.buffer_1 = Some(x);
            std::mem::swap(&mut self.buffer_0, &mut self.buffer_1);
        }
    }

    fn buffer_pop_front(&mut self) -> Option<BufferChar> {
        if let Some(x) = self.buffer_0.take() {
            std::mem::swap(&mut self.buffer_0, &mut self.buffer_1);

            Some(x)
        } else {
            None
        }
    }

    fn add_indent(&mut self) {
        if self.current_indent > 0 {
            self.buffer_push_back(BufferChar::ManySpaces);
            self.space_buffer = self.current_indent;
        }
    }
}

impl<'a> Iterator for PrettifyIter<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        // Check if there is something in the buffer to return first
        if let Some(next_char) = self.buffer_pop_front() {
            if let BufferChar::ManySpaces = next_char {
                self.space_buffer -= 1;
                if self.space_buffer > 0 {
                    // Push back to buffer to handle remaining spaces in following iterations
                    self.buffer_push_front(next_char);
                }
                return Some(' ');
            }
            return Some(next_char.into());
        }

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
                '{' | '[' if !self.in_string => {
                    self.current_indent += self.indent_width;
                    self.buffer_push_back(BufferChar::NewLine);
                    self.add_indent();
                    return Some(c);
                }
                '}' | ']' if !self.in_string => {
                    if self.current_indent >= self.indent_width {
                        self.current_indent -= self.indent_width;
                    }
                    self.add_indent();
                    self.buffer_push_back(if c == '}' {
                        BufferChar::EndObject
                    } else if c == ']' {
                        BufferChar::EndArray
                    } else {
                        unreachable!()
                    });
                    return Some('\n');
                }
                ',' if !self.in_string => {
                    self.buffer_push_back(BufferChar::NewLine);
                    self.add_indent();
                    return Some(c);
                }
                ':' if !self.in_string => {
                    // Ensure a space is added after the colon.
                    // The next character returned after the colon should be a space,
                    // ensuring that we adhere to the format "{ "key": "value" }".
                    // This involves adjusting the return statement to ensure the space
                    // is included after the colon in the output.
                    self.buffer_push_back(BufferChar::Space);
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
                _ => return Some(c), // Return the character directly for other cases.
            }
        }

        None // No more characters to return.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_prettifies_json() {
        let json = r#"{"key":"value","array":[1,2,3]}"#;
        let prettified = prettify(json, 4);

        assert_eq!(
            prettified,
            r#"{
    "key": "value",
    "array": [
        1,
        2,
        3
    ]
}"#
        );
    }

    #[test]
    fn it_handles_strings_with_escape_characters() {
        let json = r#"{"escaped": "Contains \\\"escaped\\\" characters and \"whitespace inside quotes\""}"#;
        let prettified = prettify(json, 4);

        assert_eq!(
            prettified,
            r#"{
    "escaped": "Contains \\\"escaped\\\" characters and \"whitespace inside quotes\""
}"#
        );
    }

    #[test]
    fn it_handles_very_nested_objects_and_arrays() {
        let json = r#"[[[[[[[[[{"a":"b","c":["d"]}]]]]]]]]]"#;
        let prettified = prettify(json, 4);

        assert_eq!(
            prettified,
            r#"[
    [
        [
            [
                [
                    [
                        [
                            [
                                [
                                    {
                                        "a": "b",
                                        "c": [
                                            "d"
                                        ]
                                    }
                                ]
                            ]
                        ]
                    ]
                ]
            ]
        ]
    ]
]"#
        );
    }
}
