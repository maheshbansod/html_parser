use tokenizer::{Span, Token, TokenKind, Tokenizer};

mod tokenizer;

pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        let tokenizer = Tokenizer::new(source.trim());
        Self { tokenizer }
    }

    pub fn parse(&mut self) -> Vec<Node<'a>> {
        let mut nodes = Vec::new();
        while let Some(token) = self.tokenizer.next() {
            match token.kind() {
                TokenKind::TagName { name: _ } => {
                    let attributes = self.parse_attributes();
                    let children = self.parse();
                    let element = Element {
                        attributes,
                        children,
                        tag_name: token,
                    };
                    let node = Node {
                        kind: NodeKind::Element(element),
                    };
                    nodes.push(node);
                }
                TokenKind::Text { text: _ } => {
                    let node = Node {
                        kind: NodeKind::Text(token),
                    };
                    nodes.push(node);
                }
                _ => {}
            }
        }
        nodes
    }

    fn parse_attributes(&mut self) -> Vec<Attribute<'a>> {
        let mut attributes = vec![];
        while let Some(token) = self.tokenizer.next() {
            match token.kind() {
                TokenKind::AttributeName { name: _ } => {
                    let value_token = self
                        .tokenizer
                        .next()
                        .expect("Attribute value should always exist");
                    let attribute = Attribute {
                        name: token,
                        value: value_token,
                    };
                    attributes.push(attribute);
                }
                TokenKind::OpeningTagEnd => break,
                _ => {}
            }
        }
        attributes
    }
}

#[derive(Debug)]
pub struct Node<'a> {
    kind: NodeKind<'a>,
}

#[derive(Debug)]
pub enum NodeKind<'a> {
    Text(Token<'a>),
    Element(Element<'a>),
}

#[derive(Debug)]
pub struct Element<'a> {
    attributes: Vec<Attribute<'a>>,
    children: Vec<Node<'a>>,
    tag_name: Token<'a>,
}

#[derive(Debug)]
pub struct Attribute<'a> {
    name: Token<'a>,
    value: Token<'a>,
}

impl<'a> Attribute<'a> {
    pub fn value_text(&self) -> &'a str {
        let span = self.value.span();
        let source = span.source();
        if source.starts_with('"') || source.starts_with('\'') {
            &source[1..source.len() - 1]
        } else {
            source
        }
    }
    pub fn name_text(&self) -> &'a str {
        let span = self.name.span();
        let source = span.source();
        source
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokenizer::TokenKind;

    #[test]
    fn test_basic_html_parsing() {
        let html = "<html></html>";
        let mut parser = Parser::new(html);
        let nodes = parser.parse();

        assert_eq!(nodes.len(), 1);
        match &nodes[0].kind {
            NodeKind::Element(element) => {
                assert_eq!(
                    element.tag_name.kind(),
                    &TokenKind::TagName { name: "html" }
                );
            }
            _ => panic!("Expected an element node"),
        }
    }

    #[test]
    fn test_html_with_text() {
        let html = "<html>Hello, world!</html>";
        let mut parser = Parser::new(html);
        let nodes = parser.parse();

        assert_eq!(nodes.len(), 1);
        match &nodes[0].kind {
            NodeKind::Element(element) => {
                assert_eq!(element.children.len(), 1);
                match &element.children[0].kind {
                    NodeKind::Text(text_token) => {
                        assert_eq!(
                            text_token.kind(),
                            &TokenKind::Text {
                                text: "Hello, world!"
                            }
                        );
                    }
                    _ => panic!("Expected a text node"),
                }
            }
            _ => panic!("Expected an element node"),
        }
    }

    #[test]
    fn test_html_with_attributes() {
        let html = "<html lang=\"en\"></html>";
        let mut parser = Parser::new(html);
        let nodes = parser.parse();

        assert_eq!(nodes.len(), 1);
        match &nodes[0].kind {
            NodeKind::Element(element) => {
                assert_eq!(element.attributes.len(), 1);
                assert_eq!(
                    element.attributes[0].name.kind(),
                    &TokenKind::AttributeName { name: "lang" }
                );
                assert_eq!(
                    element.attributes[0].value.kind(),
                    &TokenKind::AttributeValue { value: "en" }
                );
            }
            _ => panic!("Expected an element node"),
        }
    }

    #[test]
    fn test_empty_html() {
        let html = "";
        let mut parser = Parser::new(html);
        let nodes = parser.parse();
        assert_eq!(nodes.len(), 0);
    }

    #[test]
    fn test_unclosed_tags() {
        let html = "<html>";
        let mut parser = Parser::new(html);
        let nodes = parser.parse();
        assert_eq!(nodes.len(), 1);
        match &nodes[0].kind {
            NodeKind::Element(Element {
                attributes,
                children,
                tag_name,
            }) => {
                assert_eq!(tag_name.span().source(), "html");
                assert_eq!(children.len(), 0);
                assert_eq!(attributes.len(), 0);
            }
            _ => panic!("Expected html, got: {:?}", &nodes[0].kind),
        }
    }

    #[test]
    fn test_nested_unclosed_tags() {
        let html = "<html><div>";
        let mut parser = Parser::new(html);
        let nodes = parser.parse();
        // This might not be the correct way to handle this, but it should not panic
        assert_eq!(nodes.len(), 1);
        match &nodes[0].kind {
            NodeKind::Element(Element {
                attributes,
                children,
                tag_name,
            }) => {
                assert_eq!(tag_name.span().source(), "html");
                assert_eq!(attributes.len(), 0);
                let nodes = children;
                assert_eq!(nodes.len(), 1);
                match &nodes[0].kind {
                    NodeKind::Element(Element {
                        attributes,
                        children,
                        tag_name,
                    }) => {
                        assert_eq!(tag_name.span().source(), "div");
                        assert_eq!(attributes.len(), 0);
                        assert_eq!(children.len(), 0);
                    }
                    _ => panic!("Expected div, got: {:?}", &nodes[0].kind),
                }
            }
            _ => panic!("Expected html, got: {:?}", &nodes[0].kind),
        }
    }

    #[test]
    fn test_self_closing_tags() {
        let html = "<img src=\"example.com\"/>";
        let mut parser = Parser::new(html);
        let nodes = parser.parse();
        assert_eq!(nodes.len(), 1);
        match &nodes[0].kind {
            NodeKind::Element(element) => {
                assert_eq!(element.tag_name.kind(), &TokenKind::TagName { name: "img" });
                assert_eq!(element.attributes.len(), 1);
                assert_eq!(
                    element.attributes[0].name.kind(),
                    &TokenKind::AttributeName { name: "src" }
                );
                assert_eq!(
                    element.attributes[0].value.kind(),
                    &TokenKind::AttributeValue {
                        value: "example.com"
                    }
                );
            }
            _ => panic!("Expected an element node"),
        }
    }

    #[test]
    fn test_comments() {
        let html = "<!-- comment -->";
        let mut parser = Parser::new(html);
        let nodes = parser.parse();
        //  For now, comments are ignored
        assert_eq!(nodes.len(), 0);
    }

    #[test]
    fn test_invalid_attributes() {
        let html = "<html lang=></html>";
        let mut parser = Parser::new(html);
        let nodes = parser.parse();

        assert_eq!(nodes.len(), 1);
        match &nodes[0].kind {
            NodeKind::Element(element) => {
                assert_eq!(element.attributes.len(), 1);
                assert_eq!(
                    element.attributes[0].name.kind(),
                    &TokenKind::AttributeName { name: "lang" }
                );
                assert_eq!(
                    element.attributes[0].value.kind(),
                    &TokenKind::AttributeValue { value: "" }
                );
            }
            _ => panic!("Expected an element node"),
        }
    }

    #[test]
    fn test_special_characters() {
        let html = "<tag-name attr_name=\"attr-value\">Text with !@#$%^&*()_+=-`~[]\\{{}}|;':\",./<>?</tag-name>";
        let mut parser = Parser::new(html);
        let nodes = parser.parse();

        assert_eq!(nodes.len(), 1);
        match &nodes[0].kind {
            NodeKind::Element(element) => {
                assert_eq!(
                    element.tag_name.kind(),
                    &TokenKind::TagName { name: "tag-name" }
                );
                assert_eq!(element.attributes.len(), 1);
                assert_eq!(
                    element.attributes[0].name.kind(),
                    &TokenKind::AttributeName { name: "attr_name" }
                );
            }
            _ => panic!("Expected an element node"),
        }
    }

    #[test]
    fn test_doctype() {
        let html = "<!DOCTYPE html><html></html>";
        let mut parser = Parser::new(html);
        let nodes = parser.parse();

        assert_eq!(nodes.len(), 1);
        match &nodes[0].kind {
            NodeKind::Element(element) => {
                assert_eq!(
                    element.tag_name.kind(),
                    &TokenKind::TagName { name: "html" }
                );
            }
            _ => panic!("Expected an element node"),
        }
    }

    #[test]
    fn test_whitespace() {
        let html = "  <html  lang =  \"en\"  >  Hello  </html>  ";
        let mut parser = Parser::new(html);
        let nodes = parser.parse();

        assert_eq!(nodes.len(), 1);
        match &nodes[0].kind {
            NodeKind::Element(element) => {
                assert_eq!(
                    element.tag_name.kind(),
                    &TokenKind::TagName { name: "html" }
                );
                assert_eq!(element.attributes.len(), 1);
                assert_eq!(
                    element.attributes[0].name.kind(),
                    &TokenKind::AttributeName { name: "lang" }
                );
            }
            _ => panic!("Expected an element node"),
        }
    }

    #[test]
    fn test_case_sensitivity() {
        let html = "<HTML><Body></BODY></HTML>";
        let mut parser = Parser::new(html);
        let nodes = parser.parse();

        assert_eq!(nodes.len(), 1);
        match &nodes[0].kind {
            NodeKind::Element(element) => {
                assert_eq!(
                    element.tag_name.kind(),
                    &TokenKind::TagName { name: "HTML" }
                );
                assert_eq!(element.children.len(), 1);
                match &element.children[0].kind {
                    NodeKind::Element(body_element) => {
                        assert_eq!(
                            body_element.tag_name.kind(),
                            &TokenKind::TagName { name: "Body" }
                        );
                    }
                    _ => panic!("Expected a body element node"),
                }
            }
            _ => panic!("Expected an html element node"),
        }
    }

    #[test]
    fn test_adjacent_text_nodes() {
        let html = "<html>Hello, world!Goodbye!</html>";
        let mut parser = Parser::new(html);
        let nodes = parser.parse();

        assert_eq!(nodes.len(), 1);
        match &nodes[0].kind {
            NodeKind::Element(element) => {
                assert_eq!(element.children.len(), 1);
                match &element.children[0].kind {
                    NodeKind::Text(text_token) => {
                        assert_eq!(
                            text_token.kind(),
                            &TokenKind::Text {
                                text: "Hello, world!Goodbye!"
                            }
                        );
                    }
                    _ => panic!("Expected a text node"),
                }
            }
            _ => panic!("Expected an element node"),
        }
    }

    #[test]
    fn test_unmatched_closing_tags() {
        let html = "</html>";
        let mut parser = Parser::new(html);
        let nodes = parser.parse();
        assert_eq!(nodes.len(), 0);
    }

    #[test]
    fn test_tags_with_hyphens() {
        let html = "<custom-element></custom-element>";
        let mut parser = Parser::new(html);
        let nodes = parser.parse();

        assert_eq!(nodes.len(), 1);
        match &nodes[0].kind {
            NodeKind::Element(element) => {
                assert_eq!(
                    element.tag_name.kind(),
                    &TokenKind::TagName {
                        name: "custom-element"
                    }
                );
            }
            _ => panic!("Expected an element node"),
        }
    }

    #[test]
    fn test_attributes_with_hyphens() {
        let html = "<div data-custom=\"value\"></div>";
        let mut parser = Parser::new(html);
        let nodes = parser.parse();

        assert_eq!(nodes.len(), 1);
        match &nodes[0].kind {
            NodeKind::Element(element) => {
                assert_eq!(element.attributes.len(), 1);
                assert_eq!(
                    element.attributes[0].name.kind(),
                    &TokenKind::AttributeName {
                        name: "data-custom"
                    }
                );
                assert_eq!(
                    element.attributes[0].value.kind(),
                    &TokenKind::AttributeValue { value: "value" }
                );
            }
            _ => panic!("Expected an element node"),
        }
    }

    #[test]
    fn test_attributes_without_quotes() {
        let html = "<div data=value></div>";
        let mut parser = Parser::new(html);
        let nodes = parser.parse();

        assert_eq!(nodes.len(), 1);
        match &nodes[0].kind {
            NodeKind::Element(element) => {
                assert_eq!(element.attributes.len(), 1);
                assert_eq!(
                    element.attributes[0].name.kind(),
                    &TokenKind::AttributeName { name: "data" }
                );
                //  This is probably not correct, but it's what the current parser does
                assert_eq!(
                    element.attributes[0].value.kind(),
                    &TokenKind::AttributeValue { value: "value" }
                );
            }
            _ => panic!("Expected an element node"),
        }
    }

    #[test]
    fn test_empty_attributes() {
        let html = "<div data=\"\"></div>";
        let mut parser = Parser::new(html);
        let nodes = parser.parse();

        assert_eq!(nodes.len(), 1);
        match &nodes[0].kind {
            NodeKind::Element(element) => {
                assert_eq!(element.attributes.len(), 1);
                assert_eq!(
                    element.attributes[0].name.kind(),
                    &TokenKind::AttributeName { name: "data" }
                );
                assert_eq!(
                    element.attributes[0].value.kind(),
                    &TokenKind::AttributeValue { value: "" }
                );
            }
            _ => panic!("Expected an element node"),
        }
    }

    #[test]
    fn test_attributes_with_only_spaces() {
        let html = "<div data=\"  \"></div>";
        let mut parser = Parser::new(html);
        let nodes = parser.parse();

        assert_eq!(nodes.len(), 1);
        match &nodes[0].kind {
            NodeKind::Element(element) => {
                assert_eq!(element.attributes.len(), 1);
                assert_eq!(
                    element.attributes[0].name.kind(),
                    &TokenKind::AttributeName { name: "data" }
                );
                assert_eq!(
                    element.attributes[0].value.kind(),
                    &TokenKind::AttributeValue { value: "  " }
                );
            }
            _ => panic!("Expected an element node"),
        }
    }

    #[test]
    fn test_unicode() {
        let html = "<p>你好，世界！</p>";
        let mut parser = Parser::new(html);
        let nodes = parser.parse();

        assert_eq!(nodes.len(), 1);
        match &nodes[0].kind {
            NodeKind::Element(element) => {
                assert_eq!(element.children.len(), 1);
                match &element.children[0].kind {
                    NodeKind::Text(text_token) => {
                        assert_eq!(
                            text_token.kind(),
                            &TokenKind::Text {
                                text: "你好，世界！"
                            }
                        );
                    }
                    _ => panic!("Expected a text node"),
                }
            }
            _ => panic!("Expected an element node"),
        }
    }

    #[test]
    fn test_attr_name_value() {
        let html = "<a href=\"https://maheshbansod.com\" />";
        let mut parser = Parser::new(html);
        let nodes = parser.parse();
        assert_eq!(nodes.len(), 1);
        match &nodes[0].kind {
            NodeKind::Element(Element {
                attributes,
                children: _,
                tag_name,
            }) => {
                assert_eq!(tag_name.span().source(), "a");
                assert_eq!(attributes.len(), 1);
                assert_eq!(attributes[0].name_text(), "href");
                assert_eq!(attributes[0].value_text(), "https://maheshbansod.com");
            }
            _ => panic!("Expected an element node"),
        }
    }
}
