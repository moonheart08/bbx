//! Robust BBCode parser.
#![cfg_attr(
    not(any(feature = "track_open_tags", feature = "parser_rules")),
    doc = "This feature set is `no_std` compatible, should you want that."
)]
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
