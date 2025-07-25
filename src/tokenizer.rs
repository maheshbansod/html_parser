use std::str::CharIndices;

pub struct Tokenizer<'a> {
    source: &'a str,

    it: CharIndices<'a>,
    consume_mode: ConsumeMode,

    line: usize,
    column: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        let it = source.char_indices();
        Self {
            source,
            line: 0,
            column: 0,
            consume_mode: ConsumeMode::OutsideTag,
            it,
        }
    }

    pub fn next(&mut self) -> Option<Token<'a>> {
        match &mut self.consume_mode {
            ConsumeMode::OutsideTag => {
                if let Some(tag) = self.consume_tag() {
                    if !matches!(tag.kind, TokenKind::TagEnd { name: _ }) {
                        self.consume_mode = ConsumeMode::AttributeName;
                    }
                    Some(tag)
                } else {
                    self.consume_text_node()
                }
            }
            ConsumeMode::AttributeName => {
                self.consume_whitespace();
                self.consume_character('/');
                if let Some(tag_end) = self.consume_opening_tag_end() {
                    self.consume_mode = ConsumeMode::OutsideTag;
                    Some(tag_end)
                } else if let Some(attribute_name) = self.consume_attribute_name() {
                    self.consume_mode = ConsumeMode::AttributeValue;
                    Some(attribute_name)
                } else {
                    None
                }
            }
            ConsumeMode::AttributeValue => {
                self.consume_mode = ConsumeMode::AttributeName;
                Some(self.consume_attribute_value())
            }
        }
    }

    fn consume_attribute_value(&mut self) -> Token<'a> {
        self.consume_character('=')
            .map(|_| {
                if let Some(q) = self
                    .consume_character('"')
                    .or_else(|| self.consume_character('\''))
                {
                    let q = q
                        .source
                        .chars()
                        .next()
                        .expect("either double or single quote");
                    self.consume_characters(|c| c != &q)
                        .map(|span| {
                            self.consume_character(q);
                            let value = span.source;
                            Token {
                                span,
                                kind: TokenKind::AttributeValue { value },
                            }
                        })
                        .unwrap_or_else(|| {
                            self.consume_character(q);
                            let span = Span::point(self.current_position());
                            let value = span.source;
                            Token {
                                span,
                                kind: TokenKind::AttributeValue { value },
                            }
                        })
                } else {
                    self.consume_characters(|c| !c.is_whitespace() && c != &'>' && c != &'/')
                        .map(|span| {
                            let value = span.source;
                            Token {
                                span,
                                kind: TokenKind::AttributeValue { value },
                            }
                        })
                        .unwrap_or_else(|| {
                            let span = Span::point(self.current_position());
                            let value = span.source;
                            Token {
                                span,
                                kind: TokenKind::AttributeValue { value },
                            }
                        })
                }
            })
            .unwrap_or_else(|| {
                let span = Span::point(self.current_position());
                let value = span.source;
                Token {
                    span,
                    kind: TokenKind::AttributeValue { value },
                }
            })
    }

    fn consume_opening_tag_end(&mut self) -> Option<Token<'a>> {
        self.consume_character('>').map(|span| Token {
            span,
            kind: TokenKind::OpeningTagEnd,
        })
    }

    fn consume_text_node(&mut self) -> Option<Token<'a>> {
        self.consume_characters(|c| c != &'<').map(|text_span| {
            let text = text_span.source;
            Token {
                span: text_span,
                kind: TokenKind::Text { text },
            }
        })
    }

    fn consume_tag(&mut self) -> Option<Token<'a>> {
        let mut it_clone = self.it.clone();
        if let Some((_i, c)) = it_clone.next() {
            if c == '<' {
                // it's a tag, let's start consumption
                self.move_cursor(1);
                let is_closing = self.consume_character('/').is_some();
                let identifier = self
                    .consume_identifier()
                    .unwrap_or_else(|| Span::point(self.current_position()));
                if is_closing {
                    self.consume_character('>');
                }
                let name = identifier.source;
                Some(Token {
                    span: identifier,
                    kind: if is_closing {
                        TokenKind::TagEnd { name }
                    } else {
                        TokenKind::TagName { name }
                    },
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    fn consume_attribute_name(&mut self) -> Option<Token<'a>> {
        self.consume_identifier().map(|identifier| {
            let name = identifier.source;
            Token {
                span: identifier,
                kind: TokenKind::AttributeName { name },
            }
        })
    }

    fn consume_identifier(&mut self) -> Option<Span<'a>> {
        self.consume_whitespace();
        self.consume_characters(|c| c != &'=' && c != &'/' && c != &'>' && !c.is_whitespace())
    }

    fn current_position(&self) -> Position {
        Position {
            line: self.line,
            column: self.column,
        }
    }

    fn consume_whitespace(&mut self) {
        self.consume_characters(|c| c.is_whitespace());
    }

    fn consume_character(&mut self, c: char) -> Option<Span<'a>> {
        let start = self.current_position();
        if let Some((i, next_c)) = self.look_ahead1() {
            if next_c != c {
                return None;
            }
            let end = self.current_position();
            if c == '\n' {
                self.line += 1;
                self.column = 0;
            } else {
                self.column += 1;
            }
            self.it.next();
            Some(Span {
                range: Range { start, end },
                source: &self.source[i..i + 1],
            })
        } else {
            None
        }
    }

    /// Consume characters while the given condition evaluates to true
    fn consume_characters<F>(&mut self, condition: F) -> Option<Span<'a>>
    where
        F: Fn(&char) -> bool,
    {
        let start = Position {
            line: self.line,
            column: self.column,
        };
        let mut start_index = None;
        let mut last_index = 0;
        let mut it_clone = self.it.clone();
        while let Some((i, c)) = it_clone.next() {
            if !condition(&c) {
                break;
            }
            self.it.next();
            if start_index.is_none() {
                start_index = Some(i);
            }
            if c == '\n' {
                self.line += 1;
                self.column = 0;
            } else {
                self.column += 1;
            }
            last_index = i;
        }
        let end = Position {
            line: self.line,
            column: self.column,
        };
        start_index.map(|start_index| Span {
            range: Range { start, end },
            source: &self.source[start_index..last_index + 1],
        })
    }

    fn move_cursor(&mut self, by: usize) {
        if by == 0 {
            return;
        }
        for _ in 0..by {
            if let Some((_, c)) = self.it.next() {
                if c == '\n' {
                    self.line += 1;
                    self.column = 0;
                } else {
                    self.column += 1;
                }
            }
        }
    }

    fn look_ahead1(&mut self) -> Option<(usize, char)> {
        self.it.clone().next()
    }
}

pub struct Token<'a> {
    span: Span<'a>,
    kind: TokenKind<'a>,
}

impl<'a> Token<'a> {
    pub fn kind(&self) -> &TokenKind<'a> {
        &self.kind
    }
    pub fn span(&self) -> &Span<'a> {
        &self.span
    }
}

#[derive(Debug, PartialEq)]
pub enum TokenKind<'a> {
    TagName { name: &'a str },
    OpeningTagEnd,
    AttributeName { name: &'a str },
    AttributeValue { value: &'a str },
    Text { text: &'a str },
    TagEnd { name: &'a str },
}

enum ConsumeMode {
    AttributeName,
    AttributeValue,
    OutsideTag,
}

#[derive(Clone, Debug)]
pub struct Span<'a> {
    range: Range,
    source: &'a str,
}
impl<'a> Span<'a> {
    fn point(pos: Position) -> Self {
        Self {
            range: Range {
                start: pos.clone(),
                end: pos,
            },
            source: "",
        }
    }
}

#[derive(Clone, Debug)]
struct Range {
    start: Position,
    end: Position,
}

#[derive(Clone, Debug)]
struct Position {
    line: usize,
    column: usize,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let source = "
            <html>
            <head>
                <title>Hello</title>
            </head>
            <body>
                Welcome to my website!
            </body>
            </html>
            ";
        let mut tokenizer = Tokenizer::new(&source);
        let expected_kinds = vec![
            TokenKind::Text {
                text: "\n            ",
            },
            TokenKind::TagName { name: "html" },
            TokenKind::OpeningTagEnd,
            TokenKind::Text {
                text: "\n            ",
            },
            TokenKind::TagName { name: "head" },
            TokenKind::OpeningTagEnd,
            TokenKind::Text {
                text: "\n                ",
            },
            TokenKind::TagName { name: "title" },
            TokenKind::OpeningTagEnd,
            TokenKind::Text { text: "Hello" },
            TokenKind::TagEnd { name: "title" },
            TokenKind::Text {
                text: "\n            ",
            },
            TokenKind::TagEnd { name: "head" },
            TokenKind::Text {
                text: "\n            ",
            },
            TokenKind::TagName { name: "body" },
            TokenKind::OpeningTagEnd,
            TokenKind::Text {
                text: "\n                Welcome to my website!\n            ",
            },
            TokenKind::TagEnd { name: "body" },
            TokenKind::Text {
                text: "\n            ",
            },
            TokenKind::TagEnd { name: "html" },
            TokenKind::Text {
                text: "\n            ",
            },
        ];
        for (i, k) in expected_kinds.iter().enumerate() {
            let got_token = tokenizer
                .next()
                .expect(&format!("Token to exist. iteration: {i}"));
            assert_eq!((i, &got_token.kind), (i, k));
        }
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn attrib_basic() {
        let s = "<tag-name attr-name=attr-value>";
        let expected_kinds = vec![
            TokenKind::TagName { name: "tag-name" },
            TokenKind::AttributeName { name: "attr-name" },
            TokenKind::AttributeValue {
                value: "attr-value",
            },
            TokenKind::OpeningTagEnd,
        ];
        let mut tokenizer = Tokenizer::new(&s);
        for (i, k) in expected_kinds.iter().enumerate() {
            let got = tokenizer.next().expect("should exist");
            assert_eq!((i, &got.kind), (i, k));
        }
    }

    #[test]
    fn attrib_without_equals() {
        let s = "<tag-name attr-name>";
        let expected_kinds = vec![
            TokenKind::TagName { name: "tag-name" },
            TokenKind::AttributeName { name: "attr-name" },
            TokenKind::AttributeValue { value: "" },
            TokenKind::OpeningTagEnd,
        ];
        let mut tokenizer = Tokenizer::new(&s);
        for (i, k) in expected_kinds.into_iter().enumerate() {
            let got = tokenizer.next().map(|g| g.kind);
            assert_eq!((i, got), (i, Some(k)));
        }
    }

    #[test]
    fn attrib_quoted() {
        let s = "<tag-name attr-name=\"double quoted 'value' lets go >>> awesome\">";
        let expected_kinds = vec![
            TokenKind::TagName { name: "tag-name" },
            TokenKind::AttributeName { name: "attr-name" },
            TokenKind::AttributeValue {
                value: "double quoted 'value' lets go >>> awesome",
            },
            TokenKind::OpeningTagEnd,
        ];
        let mut tokenizer = Tokenizer::new(&s);
        for (i, k) in expected_kinds.into_iter().enumerate() {
            let got = tokenizer.next().map(|g| g.kind);
            assert_eq!((i, got), (i, Some(k)));
        }
    }

    #[test]
    fn attrib_multiple() {
        let s = "<tag-name attr-name1 attr-name2=attr-val>";
        let expected_kinds = vec![
            TokenKind::TagName { name: "tag-name" },
            TokenKind::AttributeName { name: "attr-name1" },
            TokenKind::AttributeValue { value: "" },
            TokenKind::AttributeName { name: "attr-name2" },
            TokenKind::AttributeValue { value: "attr-val" },
            TokenKind::OpeningTagEnd,
        ];
        let mut tokenizer = Tokenizer::new(&s);
        for (i, k) in expected_kinds.into_iter().enumerate() {
            let got = tokenizer.next().map(|g| g.kind);
            assert_eq!((i, got), (i, Some(k)));
        }
    }

    #[test]
    fn tag_name_with_dashes() {
        let s = "<custom-element>";
        let mut tokenizer = Tokenizer::new(&s);
        let token = tokenizer.next().expect("should exist");
        assert_eq!(
            token.kind,
            TokenKind::TagName {
                name: "custom-element"
            }
        );
    }

    #[test]
    fn empty_attribute_value_quoted() {
        let s = "<tag attr=\"\">";
        let mut tokenizer = Tokenizer::new(&s);
        tokenizer.next(); // tag
        tokenizer.next(); // attr name
        let token = tokenizer.next().expect("should exist");
        assert_eq!(token.kind, TokenKind::AttributeValue { value: "" });
        assert_eq!(
            tokenizer.next().map(|t| t.kind),
            Some(TokenKind::OpeningTagEnd)
        );
    }

    #[test]
    fn attribute_value_with_mixed_quotes() {
        let s = "<tag attr=\" He said 'hello' \">";
        let mut tokenizer = Tokenizer::new(&s);
        tokenizer.next(); // tag
        tokenizer.next(); // attr name
        let token = tokenizer.next().expect("should exist");
        assert_eq!(
            token.kind,
            TokenKind::AttributeValue {
                value: " He said 'hello' "
            }
        );
    }

    #[test]
    fn self_closing_tag() {
        let s = "<tag/>"; // not valid html but still
        let mut tokenizer = Tokenizer::new(&s);
        let tag_name = tokenizer.next().expect("should exist");
        assert_eq!(tag_name.kind, TokenKind::TagName { name: "tag" });
        let tag_end = tokenizer.next().expect("should exist");
        assert_eq!(tag_end.kind, TokenKind::OpeningTagEnd);
    }

    #[test]
    fn self_closing_tag_with_attributes() {
        let s = "<tag a b c=d/>"; // not valid html but still
        let mut tokenizer = Tokenizer::new(&s);
        let tag_name = tokenizer.next().expect("should exist");
        assert_eq!(tag_name.kind, TokenKind::TagName { name: "tag" });
        let attrib_a = tokenizer.next().expect("should exist");
        assert_eq!(attrib_a.kind, TokenKind::AttributeName { name: "a" });
        let attrib_value_a = tokenizer.next().expect("should exist");
        assert_eq!(attrib_value_a.kind, TokenKind::AttributeValue { value: "" });
        let attrib_b = tokenizer.next().expect("should exist");
        assert_eq!(attrib_b.kind, TokenKind::AttributeName { name: "b" });
        let attrib_value_b = tokenizer.next().expect("should exist");
        assert_eq!(attrib_value_b.kind, TokenKind::AttributeValue { value: "" });
        let attrib_c = tokenizer.next().expect("should exist");
        assert_eq!(attrib_c.kind, TokenKind::AttributeName { name: "c" });
        let attrib_value_c = tokenizer.next().expect("should exist");
        assert_eq!(
            attrib_value_c.kind,
            TokenKind::AttributeValue { value: "d" }
        );
        let tag_end = tokenizer.next().expect("should exist");
        assert_eq!(tag_end.kind, TokenKind::OpeningTagEnd);
    }

    #[test]
    fn attribute_name_starts_with_number() {
        let s = "<tag 1attr=value>";
        let mut tokenizer = Tokenizer::new(&s);
        tokenizer.next();
        let token = tokenizer.next().expect("should exist");
        assert_eq!(token.kind, TokenKind::AttributeName { name: "1attr" });
    }

    #[test]
    fn attribute_with_no_value_and_then_another_attribute_with_value() {
        let s = "<tag attr1 attr2=value2>";
        let mut tokenizer = Tokenizer::new(&s);

        tokenizer.next(); // tag
        assert_eq!(
            tokenizer.next().unwrap().kind,
            TokenKind::AttributeName { name: "attr1" }
        );
        assert_eq!(
            tokenizer.next().unwrap().kind,
            TokenKind::AttributeValue { value: "" }
        );
        assert_eq!(
            tokenizer.next().unwrap().kind,
            TokenKind::AttributeName { name: "attr2" }
        );
        assert_eq!(
            tokenizer.next().unwrap().kind,
            TokenKind::AttributeValue { value: "value2" }
        );
    }
}
