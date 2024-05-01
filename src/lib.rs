//! Robust BBCode parser.
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
//! # `no_std`
#![cfg_attr(
    not(any(feature = "track_open_tags", feature = "parser_rules")),
    doc = "This feature set is `no_std` compatible, should you want that."
)]
#![cfg_attr(
    any(feature = "track_open_tags", feature = "parser_rules"),
    doc = "This feature set is not `no_std` compatible but is `alloc` compatible, due to the following features:"
)]
#![cfg_attr(feature = "track_open_tags", doc = "- `track_open_tags`")]
#![cfg_attr(feature = "parser_rules", doc = "- `parser_rules`")]

#![cfg_attr(
    not(any(feature = "track_open_tags", feature = "parser_rules")),
    no_std
)]

#[cfg(any(feature = "track_open_tags", feature = "parser_rules"))]
extern crate alloc;

mod parser;

pub use parser::{BBParser, BBTag, ParserConfig, Token, TokenKind};

#[cfg(feature = "parser_rules")]
pub use parser::rules;
