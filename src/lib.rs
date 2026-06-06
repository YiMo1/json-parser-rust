use std::{fmt, iter::Peekable, str::CharIndices};

#[derive(Debug, Clone, PartialEq)]
enum JsonValue {
    String(String),
    Number(f64),
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

#[derive(Debug, Clone, PartialEq)]
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
        if let Some(value) = &self.key {
            write!(f, "key: {}, ", value)?;
        }
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
        if let Some((start, first_char)) = chars.next() {
            let mut text = String::new();
            match first_char {
                '-' | '0'..='9' => text.push(first_char),
                _ => panic!("Invalid Number"),
            }
            while let Some((pos, peek_char)) = chars.peek() {
                let (_, char) = match peek_char {
                    '.' => {
                        if text.contains('.') {
                            break;
                        }
                        let last_char = text.chars().nth_back(0).unwrap();
                        match last_char {
                            '0'..='9' => chars.next().unwrap(),
                            _ => break,
                        }
                    }
                    '0'..='9' => {
                        if !text.contains('.') {
                            let index = if first_char == '-' { 1 } else { 0 };
                            match text.chars().nth(index) {
                                Some('0') => break,
                                _ => chars.next().unwrap(),
                            }
                        } else {
                            chars.next().unwrap()
                        }
                    }
                    _ => {
                        return JsonNode {
                            name: String::from("number"),
                            key: None,
                            value: Some(JsonValue::Number(text.parse().unwrap())),
                            children: None,
                            start,
                            end: pos - 1,
                        };
                    }
                };
                text.push(char);
            }
            if chars.peek().is_none() && text.chars().nth_back(0).unwrap() != '.' {
                return JsonNode {
                    name: String::from("number"),
                    key: None,
                    value: Some(JsonValue::Number(text.parse().unwrap())),
                    children: None,
                    start,
                    end: start + text.len() - 1,
                };
            }
        }
        panic!("Invalid Number");
    }

    fn parse_array(&self, chars: &mut Peekable<CharIndices>) -> JsonNode {
        if let Some((start, '[')) = chars.next() {
            let mut node = JsonNode {
                name: String::from("array"),
                key: None,
                value: None,
                children: Some(vec![]),
                start,
                end: start,
            };
            let mut children: Vec<JsonNode> = vec![];
            self.skip_white_space(chars);
            match chars.peek() {
                Some(&(_, ']')) => {
                    let (end, _) = chars.next().unwrap();
                    node.children = Some(children);
                    node.end = end;
                    return node;
                }
                Some(_) => {
                    self.items(chars, &mut children);
                    self.skip_white_space(chars);
                    if let Some((end, ']')) = chars.next() {
                        node.children = Some(children);
                        node.end = end;
                        return node;
                    }
                }
                None => {}
            };
        }
        panic!("Invalid Array");
    }

    fn items(&self, chars: &mut Peekable<CharIndices>, children: &mut Vec<JsonNode>) {
        self.skip_white_space(chars);
        children.push(self.value(chars));
        self.skip_white_space(chars);
        if let Some((_, ',')) = chars.peek() {
            chars.next();
            self.items(chars, children);
        }
    }

    fn parse_object(&self, chars: &mut Peekable<CharIndices>) -> JsonNode {
        if let Some((start, '{')) = chars.next() {
            let mut node = JsonNode {
                name: String::from("object"),
                key: None,
                value: None,
                children: Some(vec![]),
                start,
                end: start,
            };
            let mut children: Vec<JsonNode> = vec![];
            self.skip_white_space(chars);
            match chars.peek() {
                Some(&(_, '}')) => {
                    let (end, _) = chars.next().unwrap();
                    node.children = Some(children);
                    node.end = end;
                    return node;
                }
                Some(_) => {
                    self.ky(chars, &mut children);
                    self.skip_white_space(chars);
                    if let Some((end, '}')) = chars.next() {
                        node.children = Some(children);
                        node.end = end;
                        return node;
                    }
                }
                None => {}
            };
        }
        panic!("Invalid Object");
    }

    fn ky(&self, chars: &mut Peekable<CharIndices>, children: &mut Vec<JsonNode>) {
        self.skip_white_space(chars);
        let &(start, _) = chars.peek().unwrap();
        let key = self.parse_string(chars);
        self.skip_white_space(chars);
        if let Some((_, ':')) = chars.next() {
            self.skip_white_space(chars);
            let value = self.value(chars);
            let &(end, _) = chars.peek().unwrap();
            children.push(JsonNode {
                name: String::from("proprety"),
                key: Some(Box::new(key)),
                value: Some(JsonValue::JsonNode(Box::new(value))),
                children: None,
                start,
                end,
            });
            if let Some((_, ',')) = chars.peek() {
                chars.next();
                self.ky(chars, children);
            }
        } else {
            panic!("Invalid Proprety");
        }
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
    mod object {
        use crate::*;

        fn string_node(value: &str, start: usize, end: usize) -> JsonNode {
            JsonNode {
                name: String::from("string"),
                key: None,
                value: Some(JsonValue::String(String::from(value))),
                children: None,
                start,
                end,
            }
        }

        fn number_node(value: f64, start: usize, end: usize) -> JsonNode {
            JsonNode {
                name: String::from("number"),
                key: None,
                value: Some(JsonValue::Number(value)),
                children: None,
                start,
                end,
            }
        }

        fn boolean_node(value: bool, start: usize, end: usize) -> JsonNode {
            JsonNode {
                name: String::from("boolean"),
                key: None,
                value: Some(JsonValue::Boolean(value)),
                children: None,
                start,
                end,
            }
        }

        fn null_node(start: usize, end: usize) -> JsonNode {
            JsonNode {
                name: String::from("null"),
                key: None,
                value: Some(JsonValue::Null),
                children: None,
                start,
                end,
            }
        }

        fn array_node(children: Vec<JsonNode>, start: usize, end: usize) -> JsonNode {
            JsonNode {
                name: String::from("array"),
                key: None,
                value: None,
                children: Some(children),
                start,
                end,
            }
        }

        fn object_node(children: Vec<JsonNode>, start: usize, end: usize) -> JsonNode {
            JsonNode {
                name: String::from("object"),
                key: None,
                value: None,
                children: Some(children),
                start,
                end,
            }
        }

        fn property_node(key: JsonNode, value: JsonNode, start: usize, end: usize) -> JsonNode {
            JsonNode {
                name: String::from("proprety"),
                key: Some(Box::new(key)),
                value: Some(JsonValue::JsonNode(Box::new(value))),
                children: None,
                start,
                end,
            }
        }

        #[test]
        fn empty() {
            assert_eq!(JsonParser::new().parse("{}"), object_node(vec![], 0, 1));
        }

        #[test]
        fn empty_with_whitespace() {
            assert_eq!(
                JsonParser::new().parse(" \n{ \t}\r"),
                object_node(vec![], 2, 5)
            );
        }

        #[test]
        fn single_property() {
            assert_eq!(
                JsonParser::new().parse("{\"name\":\"codex\"}"),
                object_node(
                    vec![property_node(
                        string_node("name", 1, 6),
                        string_node("codex", 8, 14),
                        1,
                        15,
                    )],
                    0,
                    15,
                )
            );
        }

        #[test]
        fn mixed_scalars() {
            assert_eq!(
                JsonParser::new().parse("{\"n\":1,\"b\":false,\"x\":null}"),
                object_node(
                    vec![
                        property_node(string_node("n", 1, 3), number_node(1.0, 5, 5), 1, 6),
                        property_node(string_node("b", 7, 9), boolean_node(false, 11, 15), 7, 16,),
                        property_node(string_node("x", 17, 19), null_node(21, 24), 17, 25,),
                    ],
                    0,
                    25,
                )
            );
        }

        #[test]
        fn nested_values() {
            assert_eq!(
                JsonParser::new().parse("{\"arr\":[1],\"obj\":{\"ok\":true}}"),
                object_node(
                    vec![
                        property_node(
                            string_node("arr", 1, 5),
                            array_node(vec![number_node(1.0, 8, 8)], 7, 9),
                            1,
                            10,
                        ),
                        property_node(
                            string_node("obj", 11, 15),
                            object_node(
                                vec![property_node(
                                    string_node("ok", 18, 21),
                                    boolean_node(true, 23, 26),
                                    18,
                                    27,
                                )],
                                17,
                                27,
                            ),
                            11,
                            28,
                        ),
                    ],
                    0,
                    28,
                )
            );
        }

        #[test]
        #[should_panic]
        fn missing_colon_is_invalid() {
            JsonParser::new().parse("{\"a\" 1}");
        }

        #[test]
        #[should_panic]
        fn missing_closing_brace_is_invalid() {
            JsonParser::new().parse("{\"a\":1");
        }

        #[test]
        #[should_panic]
        fn trailing_comma_is_invalid() {
            JsonParser::new().parse("{\"a\":1,}");
        }

        #[test]
        #[should_panic]
        fn missing_comma_is_invalid() {
            JsonParser::new().parse("{\"a\":1 \"b\":2}");
        }
    }

    mod number {
        use crate::*;

        fn number_node(value: f64, start: usize, end: usize) -> JsonNode {
            JsonNode {
                name: String::from("number"),
                key: None,
                value: Some(JsonValue::Number(value)),
                children: None,
                start,
                end,
            }
        }

        #[test]
        fn float() {
            assert_eq!(
                JsonParser::new().parse("123.123"),
                number_node(123.123, 0, 6)
            );
        }

        #[test]
        fn integer() {
            assert_eq!(JsonParser::new().parse("123"), number_node(123.0, 0, 2));
        }

        #[test]
        fn float_zero() {
            assert_eq!(JsonParser::new().parse("0.0"), number_node(0.0, 0, 2));
        }

        #[test]
        fn negative() {
            assert_eq!(
                JsonParser::new().parse("-123.123"),
                number_node(-123.123, 0, 7)
            );
        }

        #[test]
        fn zero_integer() {
            assert_eq!(JsonParser::new().parse("0"), number_node(0.0, 0, 0));
        }

        #[test]
        fn leading_and_trailing_whitespace() {
            assert_eq!(
                JsonParser::new().parse(" \t123\n"),
                number_node(123.0, 2, 4)
            );
        }

        #[test]
        fn negative_zero() {
            assert_eq!(JsonParser::new().parse("-0"), number_node(-0.0, 0, 1));
        }

        #[test]
        fn negative_fraction() {
            assert_eq!(JsonParser::new().parse("-0.5"), number_node(-0.5, 0, 3));
        }

        #[test]
        #[should_panic]
        fn leading_zero_integer_is_invalid() {
            JsonParser::new().parse("01");
        }

        #[test]
        #[should_panic]
        fn trailing_dot_is_invalid() {
            JsonParser::new().parse("1.");
        }

        #[test]
        #[should_panic]
        fn multiple_dots_is_invalid() {
            JsonParser::new().parse("1.2.3");
        }

        #[test]
        #[should_panic]
        fn minus_without_digits_is_invalid() {
            JsonParser::new().parse("-");
        }
    }

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

    mod array {
        use crate::*;

        fn string_node(value: &str, start: usize, end: usize) -> JsonNode {
            JsonNode {
                name: String::from("string"),
                key: None,
                value: Some(JsonValue::String(String::from(value))),
                children: None,
                start,
                end,
            }
        }

        fn number_node(value: f64, start: usize, end: usize) -> JsonNode {
            JsonNode {
                name: String::from("number"),
                key: None,
                value: Some(JsonValue::Number(value)),
                children: None,
                start,
                end,
            }
        }

        fn boolean_node(value: bool, start: usize, end: usize) -> JsonNode {
            JsonNode {
                name: String::from("boolean"),
                key: None,
                value: Some(JsonValue::Boolean(value)),
                children: None,
                start,
                end,
            }
        }

        fn null_node(start: usize, end: usize) -> JsonNode {
            JsonNode {
                name: String::from("null"),
                key: None,
                value: Some(JsonValue::Null),
                children: None,
                start,
                end,
            }
        }

        fn array_node(children: Vec<JsonNode>, start: usize, end: usize) -> JsonNode {
            JsonNode {
                name: String::from("array"),
                key: None,
                value: None,
                children: Some(children),
                start,
                end,
            }
        }

        #[test]
        fn empty() {
            assert_eq!(JsonParser::new().parse("[]"), array_node(vec![], 0, 1));
        }

        #[test]
        fn empty_with_whitespace() {
            assert_eq!(
                JsonParser::new().parse(" \n[ \t]\r"),
                array_node(vec![], 2, 5)
            );
        }

        #[test]
        fn mixed_scalars() {
            assert_eq!(
                JsonParser::new().parse("[\"hi\", 1, true, null]"),
                array_node(
                    vec![
                        string_node("hi", 1, 4),
                        number_node(1.0, 7, 7),
                        boolean_node(true, 10, 13),
                        null_node(16, 19),
                    ],
                    0,
                    20,
                )
            );
        }

        #[test]
        fn nested_array() {
            assert_eq!(
                JsonParser::new().parse("[[], [1]]"),
                array_node(
                    vec![
                        array_node(vec![], 1, 2),
                        array_node(vec![number_node(1.0, 6, 6)], 5, 7),
                    ],
                    0,
                    8,
                )
            );
        }

        #[test]
        #[should_panic]
        fn missing_closing_bracket_is_invalid() {
            JsonParser::new().parse("[1, 2");
        }

        #[test]
        #[should_panic]
        fn trailing_comma_is_invalid() {
            JsonParser::new().parse("[1,]");
        }

        #[test]
        #[should_panic]
        fn missing_comma_is_invalid() {
            JsonParser::new().parse("[1 2]");
        }
    }
}
