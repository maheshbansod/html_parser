
# html-parser

A simple HTML parser written in Rust.

## Description

`html-parser` is a basic HTML parser implemented in Rust. It tokenizes and parses HTML-like input into a simplified DOM representation. This parser is not fully spec-compliant and may handle certain edge cases in an unconventional manner. It's primarily intended for personal use and experimentation.

## Usage

```rust
use html_parser::Parser;

fn main() {
    let html = "<html><body><h1>Hello, world!</h1></body></html>";
    let mut parser = Parser::new(html);
    let nodes = parser.parse();

    println!("{:?}", nodes);
}
```

The `parse` method returns a `Vec<Node>`, where `Node` represents an element, text, or comment in the HTML structure.

## Features

*   Tokenization of HTML-like input
*   Basic DOM tree construction
*   Handles nested tags
*   Parses attributes (with and without quotes)
*   Support for self-closing tags

## Limitations

*   Not fully spec-compliant
*   Error handling is basic
*   Performance has not been optimized
*   Doesn't support all HTML features (e.g., CDATA sections, some entity encodings)
*   Comments are ignored

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
