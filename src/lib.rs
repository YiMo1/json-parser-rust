use std::{fmt, iter::Peekable, str::CharIndices};

#[derive(Debug, Clone, PartialEq, Eq)]
enum JsonValue {
    String(String),
    Number(isize),
    Null,
    Boolean(bool),
    JsonNode(Box<JsonNode>),
}

impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JsonValue::Boolean(bool) => write!(f, "{}", bool),
            JsonValue::String(string) => write!(f, "{}", string),
            JsonValue::Number(number) => write!(f, "{}", number),
            JsonValue::Null => write!(f, "null"),
            JsonValue::JsonNode(node) => write!(f, "{}", node),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonNode {
    name: String,
    key: Option<Box<JsonNode>>,
    value: Option<JsonValue>,
    children: Option<Vec<JsonNode>>,
    start: usize,
    end: usize,
}

impl fmt::Display for JsonNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ name: {}, ", self.name)?;
        if let Some(value) = &self.value {
            write!(f, "value: {}, ", value)?;
        }
        if let Some(children) = &self.children {
            write!(f, "[")?;
            for (index, item) in children.iter().enumerate() {
                if index > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", item)?;
            }
            write!(f, "], ")?;
        }
        write!(f, "start: {}, ", self.start)?;
        write!(f, "end: {} }}", self.end)
    }
}

pub struct JsonParser {}

impl JsonParser {
    pub fn new() -> Self {
        return JsonParser {};
    }

    pub fn parse(&self, json_str: &str) -> JsonNode {
        let mut chars = json_str.char_indices().peekable();
        self.skip_white_space(&mut chars);
        let node = self.value(&mut chars);
        self.skip_white_space(&mut chars);
        if let Some(_) = chars.next() {
            panic!("传入的字符串不是一个合法的JSON字符");
        }
        node
    }

    fn parse_string(&self, chars: &mut Peekable<CharIndices>) -> JsonNode {
        if let Some((start, '"')) = chars.next() {
            let mut text = String::new();
            let mut translation_flag = false;
            while let Some((end, char)) = chars.next() {
                if translation_flag {
                    text.push(char);
                    translation_flag = false;
                    continue;
                }

                if char == '\\' {
                    translation_flag = true;
                    text.push(char);
                    continue;
                }

                if char != '"' {
                    text.push(char);
                    continue;
                }

                return JsonNode {
                    name: String::from("string"),
                    key: None,
                    value: Some(JsonValue::String(text)),
                    children: None,
                    start,
                    end,
                };
            }
            panic!("Invalid String")
        } else {
            panic!("Invalid String")
        }
    }

    fn parse_null(&self, chars: &mut Peekable<CharIndices>) -> JsonNode {
        let mut text = String::new();
        while let Some((pos, char)) = chars.next() {
            text.push(char);
            if !"null".starts_with(&text) {
                break;
            };

            if text == "null" {
                return JsonNode {
                    name: String::from("null"),
                    key: None,
                    value: Some(JsonValue::Null),
                    children: None,
                    start: pos - 3,
                    end: pos,
                };
            }
        }

        panic!("Invalid Null");
    }

    fn parse_number(&self, chars: &mut Peekable<CharIndices>) -> JsonNode {
        todo!()
    }

    fn parse_array(&self, chars: &mut Peekable<CharIndices>) -> JsonNode {
        todo!()
    }

    fn parse_object(&self, chars: &mut Peekable<CharIndices>) -> JsonNode {
        todo!()
    }

    fn parse_boolean(&self, chars: &mut Peekable<CharIndices>) -> JsonNode {
        let mut text = String::new();
        while let Some((pos, char)) = chars.next() {
            text.push(char);
            if !"true".starts_with(&text) && !"false".starts_with(&text) {
                break;
            };

            if text == "true" {
                return JsonNode {
                    name: String::from("boolean"),
                    key: None,
                    value: Some(JsonValue::Boolean(true)),
                    children: None,
                    start: pos - 3,
                    end: pos,
                };
            }

            if text == "false" {
                return JsonNode {
                    name: String::from("boolean"),
                    key: None,
                    value: Some(JsonValue::Boolean(false)),
                    children: None,
                    start: pos - 4,
                    end: pos,
                };
            }
        }

        panic!("Invalid Boolean");
    }

    fn skip_white_space(&self, chars: &mut Peekable<CharIndices>) {
        while let Some((_, char)) = chars.peek() {
            match char {
                ' ' | '\t' | '\n' | '\r' => {
                    chars.next();
                }
                _ => break,
            }
        }
    }

    fn value(&self, chars: &mut Peekable<CharIndices>) -> JsonNode {
        match chars.peek() {
            Some((_, char)) => match char {
                '"' => self.parse_string(chars),
                'n' => self.parse_null(chars),
                '{' => self.parse_object(chars),
                '[' => self.parse_array(chars),
                't' | 'f' => self.parse_boolean(chars),
                '-' | '0'..='9' => self.parse_number(chars),
                _ => panic!("Invalid Value"),
            },
            None => panic!("Invalid Value"),
        }
    }
}

#[cfg(test)]
mod tests {
    mod boolean {
        use crate::*;

        #[test]
        fn normal() {
            assert_eq!(
                JsonParser::new().parse("true"),
                JsonNode {
                    name: String::from("boolean"),
                    key: None,
                    value: Some(JsonValue::Boolean(true)),
                    children: None,
                    start: 0,
                    end: 3
                }
            );
            assert_eq!(
                JsonParser::new().parse("false"),
                JsonNode {
                    name: String::from("boolean"),
                    key: None,
                    value: Some(JsonValue::Boolean(false)),
                    children: None,
                    start: 0,
                    end: 4
                }
            );
        }
    }

    mod null {
        use crate::*;
        #[test]
        fn normal() {
            assert_eq!(
                JsonParser::new().parse("null"),
                JsonNode {
                    name: String::from("null"),
                    value: Some(JsonValue::Null),
                    key: None,
                    children: None,
                    start: 0,
                    end: 3
                }
            );
        }
    }

    mod string {
        use crate::*;

        #[test]
        fn empty() {
            // ""
            assert_eq!(
                JsonParser::new().parse("\"\""),
                JsonNode {
                    start: 0,
                    end: 1,
                    value: Some(JsonValue::String(String::from(""))),
                    name: String::from("string"),
                    key: None,
                    children: None
                },
            );
        }

        #[test]
        fn normal() {
            // "hello, world"
            assert_eq!(
                JsonParser::new().parse("\"hello, world\""),
                JsonNode {
                    start: 0,
                    end: 13,
                    value: Some(JsonValue::String(String::from("hello, world"))),
                    name: String::from("string"),
                    key: None,
                    children: None
                }
            );
        }

        #[test]
        fn both_whitespace() {
            //  \t\n\r"hello, world" \t\n\r
            assert_eq!(
                JsonParser::new().parse(" \t\n\r\"hello, world\" \t\n\r"),
                JsonNode {
                    start: 4,
                    end: 17,
                    value: Some(JsonValue::String(String::from("hello, world"))),
                    name: String::from("string"),
                    key: None,
                    children: None
                }
            );
        }

        #[test]
        fn translate_char() {
            // "\\\""
            assert_eq!(
                JsonParser::new().parse("\"\\\\\\\"\""),
                JsonNode {
                    start: 0,
                    end: 5,
                    value: Some(JsonValue::String(String::from("\\\\\\\""))),
                    name: String::from("string"),
                    key: None,
                    children: None
                }
            );
        }
    }
}
