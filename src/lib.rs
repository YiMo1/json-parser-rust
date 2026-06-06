use std::{fmt, iter::Peekable, str::CharIndices};

#[derive(Debug, Clone, PartialEq)]
enum TokenKind {
    String,
    Number,
    Null,
    Boolean,
    Object,
    Array,
    Property,
}

#[derive(Debug, Clone, PartialEq)]
enum JsonValue {
    String(String),
    JsonNode(Box<JsonNode>),
}

impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JsonValue::String(string) => write!(f, "{}", string),
            JsonValue::JsonNode(node) => write!(f, "{}", node),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsonNode {
    kind: TokenKind,
    key: Option<Box<JsonNode>>,
    value: Option<JsonValue>,
    children: Option<Vec<JsonNode>>,
    start: usize,
    end: usize,
}

impl fmt::Display for JsonNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ kind: {}, ",
            match self.kind {
                TokenKind::Array => "Array",
                TokenKind::String => "String",
                TokenKind::Number => "Number",
                TokenKind::Null => "Null",
                TokenKind::Boolean => "Boolean",
                TokenKind::Object => "Object",
                TokenKind::Property => "Property",
            }
        )?;
        if let Some(value) = &self.key {
            write!(f, "key: {}, ", value)?;
        }
        if let Some(value) = &self.value {
            write!(f, "value: {}, ", value)?;
        }
        if let Some(children) = &self.children {
            write!(f, "children: [")?;
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
                    kind: TokenKind::String,
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
                    kind: TokenKind::Null,
                    key: None,
                    value: Some(JsonValue::String(text)),
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
                            kind: TokenKind::Number,
                            key: None,
                            value: Some(JsonValue::String(text)),
                            children: None,
                            start,
                            end: pos - 1,
                        };
                    }
                };
                text.push(char);
            }
            if chars.peek().is_none()
                && text.chars().nth_back(0).unwrap() != '.'
                && text.chars().nth_back(0).unwrap() != '-'
            {
                let end = start + text.len() - 1;
                return JsonNode {
                    kind: TokenKind::Number,
                    key: None,
                    value: Some(JsonValue::String(text)),
                    children: None,
                    start,
                    end,
                };
            }
        }
        panic!("Invalid Number");
    }

    fn parse_array(&self, chars: &mut Peekable<CharIndices>) -> JsonNode {
        if let Some((start, '[')) = chars.next() {
            let mut node = JsonNode {
                kind: TokenKind::Array,
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
                kind: TokenKind::Object,
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
                kind: TokenKind::Property,
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

            if text == "true" || text == "false" {
                let start = pos + 1 - text.len();
                return JsonNode {
                    kind: TokenKind::Boolean,
                    key: None,
                    value: Some(JsonValue::String(text)),
                    children: None,
                    start,
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
    use crate::*;

    fn scalar_node(kind: TokenKind, value: &str, start: usize, end: usize) -> JsonNode {
        JsonNode {
            kind,
            key: None,
            value: Some(JsonValue::String(String::from(value))),
            children: None,
            start,
            end,
        }
    }

    fn string_node(value: &str, start: usize, end: usize) -> JsonNode {
        scalar_node(TokenKind::String, value, start, end)
    }

    fn number_node(value: &str, start: usize, end: usize) -> JsonNode {
        scalar_node(TokenKind::Number, value, start, end)
    }

    fn boolean_node(value: &str, start: usize, end: usize) -> JsonNode {
        scalar_node(TokenKind::Boolean, value, start, end)
    }

    fn null_node(start: usize, end: usize) -> JsonNode {
        scalar_node(TokenKind::Null, "null", start, end)
    }

    fn array_node(children: Vec<JsonNode>, start: usize, end: usize) -> JsonNode {
        JsonNode {
            kind: TokenKind::Array,
            key: None,
            value: None,
            children: Some(children),
            start,
            end,
        }
    }

    fn object_node(children: Vec<JsonNode>, start: usize, end: usize) -> JsonNode {
        JsonNode {
            kind: TokenKind::Object,
            key: None,
            value: None,
            children: Some(children),
            start,
            end,
        }
    }

    fn property_node(key: JsonNode, value: JsonNode, start: usize, end: usize) -> JsonNode {
        JsonNode {
            kind: TokenKind::Property,
            key: Some(Box::new(key)),
            value: Some(JsonValue::JsonNode(Box::new(value))),
            children: None,
            start,
            end,
        }
    }

    fn text_value(node: &JsonNode) -> &str {
        match node.value.as_ref() {
            Some(JsonValue::String(value)) => value,
            _ => panic!("expected string value"),
        }
    }

    fn nested_value(node: &JsonNode) -> &JsonNode {
        match node.value.as_ref() {
            Some(JsonValue::JsonNode(value)) => value,
            _ => panic!("expected nested node"),
        }
    }

    fn children(node: &JsonNode) -> &[JsonNode] {
        node.children.as_deref().expect("expected children")
    }

    mod string {
        use super::*;

        #[test]
        fn empty() {
            assert_eq!(JsonParser::new().parse("\"\""), string_node("", 0, 1));
        }

        #[test]
        fn normal() {
            assert_eq!(
                JsonParser::new().parse("\"hello, world\""),
                string_node("hello, world", 0, 13)
            );
        }

        #[test]
        fn both_whitespace() {
            assert_eq!(
                JsonParser::new().parse(" \t\n\r\"hello, world\" \t\n\r"),
                string_node("hello, world", 4, 17)
            );
        }

        #[test]
        fn escaped_characters_are_preserved() {
            assert_eq!(
                JsonParser::new().parse("\"\\\\\\\"\""),
                string_node("\\\\\\\"", 0, 5)
            );
        }

        #[test]
        fn punctuation_inside_string() {
            assert_eq!(
                JsonParser::new().parse("\"[1, 2]: {ok}\""),
                string_node("[1, 2]: {ok}", 0, 13)
            );
        }

        #[test]
        #[should_panic]
        fn unclosed_string_is_invalid() {
            JsonParser::new().parse("\"unterminated");
        }
    }

    mod number {
        use super::*;

        #[test]
        fn integer() {
            assert_eq!(JsonParser::new().parse("123"), number_node("123", 0, 2));
        }

        #[test]
        fn float() {
            assert_eq!(
                JsonParser::new().parse("123.123"),
                number_node("123.123", 0, 6)
            );
        }

        #[test]
        fn zero_integer() {
            assert_eq!(JsonParser::new().parse("0"), number_node("0", 0, 0));
        }

        #[test]
        fn negative_integer() {
            assert_eq!(JsonParser::new().parse("-42"), number_node("-42", 0, 2));
        }

        #[test]
        fn negative_fraction() {
            assert_eq!(JsonParser::new().parse("-0.5"), number_node("-0.5", 0, 3));
        }

        #[test]
        fn leading_and_trailing_whitespace() {
            assert_eq!(
                JsonParser::new().parse(" \t123\n"),
                number_node("123", 2, 4)
            );
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
        fn exponent_notation_is_invalid() {
            JsonParser::new().parse("1e3");
        }

        #[test]
        #[should_panic]
        fn minus_without_digits_is_invalid() {
            JsonParser::new().parse("-");
        }
    }

    mod boolean {
        use super::*;

        #[test]
        fn false_literal() {
            let false_node = JsonParser::new().parse("false");
            assert_eq!(false_node.kind, TokenKind::Boolean);
            assert_eq!(text_value(&false_node), "false");
            assert_eq!(false_node.start, 0);
            assert_eq!(false_node.end, 4);
        }

        #[test]
        fn true_literal() {
            let true_node = JsonParser::new().parse("true");
            assert_eq!(true_node.kind, TokenKind::Boolean);
            assert_eq!(text_value(&true_node), "true");
            assert_eq!(true_node.start, 0);
            assert_eq!(true_node.end, 3);
        }

        #[test]
        fn surrounding_whitespace() {
            let node = JsonParser::new().parse(" \tfalse\n");
            assert_eq!(node.kind, TokenKind::Boolean);
            assert_eq!(text_value(&node), "false");
            assert_eq!(node.end, 6);
        }

        #[test]
        #[should_panic]
        fn mixed_case_is_invalid() {
            JsonParser::new().parse("True");
        }

        #[test]
        #[should_panic]
        fn trailing_identifier_is_invalid() {
            JsonParser::new().parse("trueish");
        }
    }

    mod null {
        use super::*;

        #[test]
        fn normal() {
            assert_eq!(JsonParser::new().parse("null"), null_node(0, 3));
        }

        #[test]
        fn surrounding_whitespace() {
            assert_eq!(JsonParser::new().parse(" \rnull\t"), null_node(2, 5));
        }

        #[test]
        #[should_panic]
        fn invalid_suffix() {
            JsonParser::new().parse("nullish");
        }
    }

    mod array {
        use super::*;

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
                JsonParser::new().parse("[\"hi\", 1, null]"),
                array_node(
                    vec![
                        string_node("hi", 1, 4),
                        number_node("1", 7, 7),
                        null_node(10, 13),
                    ],
                    0,
                    14,
                )
            );
        }

        #[test]
        fn boolean_items() {
            let node = JsonParser::new().parse("[true,false]");
            let items = children(&node);
            assert_eq!(node.kind, TokenKind::Array);
            assert_eq!(node.start, 0);
            assert_eq!(node.end, 11);
            assert_eq!(items.len(), 2);
            assert_eq!(items[0], boolean_node("true", 1, 4));
            assert_eq!(items[1], boolean_node("false", 6, 10));
        }

        #[test]
        fn nested_array() {
            assert_eq!(
                JsonParser::new().parse("[[], [1, 2]]"),
                array_node(
                    vec![
                        array_node(vec![], 1, 2),
                        array_node(vec![number_node("1", 6, 6), number_node("2", 9, 9)], 5, 10),
                    ],
                    0,
                    11,
                )
            );
        }

        #[test]
        fn nested_object() {
            assert_eq!(
                JsonParser::new().parse("[{\"a\":\"b\"}]"),
                array_node(
                    vec![object_node(
                        vec![property_node(
                            string_node("a", 2, 4),
                            string_node("b", 6, 8),
                            2,
                            9,
                        )],
                        1,
                        9,
                    )],
                    0,
                    10,
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

        #[test]
        #[should_panic]
        fn leading_comma_is_invalid() {
            JsonParser::new().parse("[,1]");
        }
    }

    mod object {
        use super::*;

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
                JsonParser::new().parse("{\"s\":\"v\",\"n\":10,\"x\":null}"),
                object_node(
                    vec![
                        property_node(string_node("s", 1, 3), string_node("v", 5, 7), 1, 8),
                        property_node(string_node("n", 9, 11), number_node("10", 13, 14), 9, 15),
                        property_node(string_node("x", 16, 18), null_node(20, 23), 16, 24),
                    ],
                    0,
                    24,
                )
            );
        }

        #[test]
        fn boolean_property() {
            let node = JsonParser::new().parse("{\"ok\":true}");
            let props = children(&node);
            assert_eq!(node.kind, TokenKind::Object);
            assert_eq!(node.start, 0);
            assert_eq!(node.end, 10);
            assert_eq!(props.len(), 1);
            assert_eq!(props[0].kind, TokenKind::Property);
            assert_eq!(props[0].start, 1);
            assert_eq!(props[0].end, 10);
            assert_eq!(props[0].key.as_deref(), Some(&string_node("ok", 1, 4)));
            assert_eq!(nested_value(&props[0]), &boolean_node("true", 6, 9));
        }

        #[test]
        fn nested_values() {
            assert_eq!(
                JsonParser::new().parse("{\"arr\":[1,2],\"obj\":{\"k\":\"v\"}}"),
                object_node(
                    vec![
                        property_node(
                            string_node("arr", 1, 5),
                            array_node(
                                vec![number_node("1", 8, 8), number_node("2", 10, 10)],
                                7,
                                11
                            ),
                            1,
                            12,
                        ),
                        property_node(
                            string_node("obj", 13, 17),
                            object_node(
                                vec![property_node(
                                    string_node("k", 20, 22),
                                    string_node("v", 24, 26),
                                    20,
                                    27,
                                )],
                                19,
                                27,
                            ),
                            13,
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

        #[test]
        #[should_panic]
        fn bare_identifier_key_is_invalid() {
            JsonParser::new().parse("{a:1}");
        }
    }
}
