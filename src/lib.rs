//! Robust BBCode parser with support for emulating legacy parsers, complex parsing extensions, .

#![cfg_attr(test, feature(assert_matches))]

mod parser;

pub use parser::{BBParser, ParserConfig, Token, TokenKind};

#[cfg(feature = "parser_rules")]
pub use parser::rules;
