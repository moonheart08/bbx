//! Robust and performant BBCode pull parser.
//! # Examples
//! ## Simple parsing
//! ```rust
//! # use bbx::BBParser;
//! # const input: &str = "This would be lorem ipsum [b]but doing so is kind of unnecessary[/b]";
//! let mut parser = BBParser::new(/* &str */ input);
//!
//! for token in parser {
//!     println!("{:?}", token);
//! }
//! ```
//! ## Built-in sanitized HTML output
//! ```rust
//! # use bbx::{BBParser, html::{*, builtins::*}};
//! # const input: &str = "[title]This is a test document![/title]";
//! // Simple serializer default with all of the v1.0.0 tags considered "core" to the library.
//! let mut serializer: HtmlSerializer<SimpleHtmlWriter> = 
//!     HtmlSerializer::with_tags(all_core_v1_tags());
//! let mut parser = BBParser::new(input);
//! println!("Document:");
//! println!("{}", serializer.serialize(parser));
//! ```
//! # `no_std`
#![cfg_attr(
    not(feature = "alloc"),
    doc = "This feature set is `no_std` compatible, should you want that."
)]
#![cfg_attr(
    all(feature = "alloc", not(feature = "html_gen")),
    doc = "This feature set is not `no_std` compatible but is `alloc` compatible, due to the following features:"
)]
#![cfg_attr(
    all(feature = "alloc", feature = "html_gen"),
    doc = "This feature set is `std` (hosted) only, due to the following features:"
)]
#![cfg_attr(feature = "track_open_tags", doc = "- `track_open_tags`")]
#![cfg_attr(feature = "parser_rules", doc = "- `parser_rules`")]
#![cfg_attr(feature = "html_gen", doc = "- `html_gen` (required `std`!)")]
#![cfg_attr(not(all(feature = "alloc", feature = "html_gen")), no_std)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

extern crate alloc;

mod parser;

pub use parser::{BBParser, BBTag, ParserConfig, Token, TokenKind};

#[cfg(feature = "parser_rules")]
pub use parser::rules;

#[cfg(feature = "html_gen")]
pub mod html;
