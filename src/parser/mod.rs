#[cfg(any(feature = "track_open_tags", feature = "parser_rules"))]
use alloc::vec;
use core::marker::PhantomData;
use static_assertions::{assert_impl_all, assert_not_impl_all};

use bitflags::bitflags;

/// Provides configuration information for [BBParser], including enabled feature flags.
#[derive(Default)]
pub struct ParserConfig {
    /// Feature flags for this configuration.
    pub feature_flags: ParserFeature,
}

bitflags! {
    /// Represents a set of flags.
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ParserFeature: u32 {
        /// Allow tags to be removed out of order.
        const POP_UNORDERED = 1 << 1;

        /// Textualize close tags with no corresponding open tag, instead of preserving the token.
        const UNMATCHED_CLOSE_AS_TEXT = 1 << 2;

        /// All compatibility features in v1.0.0 and earlier.
        const V1 = Self::POP_UNORDERED.bits() | Self::UNMATCHED_CLOSE_AS_TEXT.bits();

        /// All current and future feature flags.
        /// Prefer using one of the versioned versions of this mask (i.e. V1) if you have stability guarantees.
        const ALL = u32::MAX;
    }
}

/// Provides a BBCode parser over the given input, in the form of an iterator.
/// BBParser is a *pull parser*, parsing the input on an on-demand basis as the user calls [BBParser::next].
/// # Allocations
#[cfg_attr(
    not(any(feature = "track_open_tags", feature = "parser_rules")),
    doc = "BBParser does not allocate on the current configuration."
)]
#[cfg_attr(
    any(feature = "track_open_tags", feature = "parser_rules"),
    doc = "BBParser allocates on the current configuration when:"
)]
#[cfg_attr(
    feature = "track_open_tags",
    doc = "- An opening tag is encountered. (`track_open_tags`)"
)]
#[cfg_attr(
    feature = "parser_rules",
    doc = "- A parser rule is inserted. (`parser_rules`)"
)]
#[doc(alias = "parser")]
pub struct BBParser<'a, CustomTy = ()>
where
    CustomTy: Clone,
{
    input: &'a str,
    config: ParserConfig,
    loc: usize,
    #[cfg(feature = "track_open_tags")]
    open_tags: Vec<Token<'a, CustomTy>>,
    #[cfg(feature = "track_open_tags")]
    closed_tags: Vec<Token<'a, CustomTy>>,
    #[cfg(feature = "parser_rules")]
    rule_stack: Vec<Box<dyn rules::ParserRuleInner<'a, CustomTy> + Send + 'a>>,
    _custom_ty: PhantomData<CustomTy>,
}

assert_impl_all!(BBParser<'_, ()>: Send);
assert_not_impl_all!(BBParser<'_, *const u8>: Send);

/// Standard constructors.
impl<'a> BBParser<'a> {
    /// Constructs a new parser for the given input string, using the default [ParserConfig].
    pub fn new(input: &'a str) -> BBParser<'a> {
        Self::new_with_custom(input)
    }

    /// Constructs a new parser for the given input string and configuration.
    pub fn with_config(input: &'a str, config: ParserConfig) -> BBParser<'a> {
        Self::with_config_and_custom(input, config)
    }
}

/// Custom token type compatible constructors.
impl<'a> BBParser<'a> {
    /// Constructs a new parser for the given input string, using the default [ParserConfig].
    pub fn new_with_custom<CustomTy>(input: &'a str) -> BBParser<'a, CustomTy>
    where
        CustomTy: Clone,
    {
        BBParser::<'a, CustomTy> {
            input,
            config: Default::default(),
            loc: 0,
            #[cfg(feature = "track_open_tags")]
            open_tags: vec![],
            #[cfg(feature = "track_open_tags")]
            closed_tags: vec![],
            #[cfg(feature = "parser_rules")]
            rule_stack: vec![],
            _custom_ty: PhantomData,
        }
    }

    /// Constructs a new parser for the given input string and configuration.
    pub fn with_config_and_custom<CustomTy>(
        input: &'a str,
        config: ParserConfig,
    ) -> BBParser<'a, CustomTy>
    where
        CustomTy: Clone,
    {
        BBParser::<'a, CustomTy> {
            input,
            config,
            loc: 0,
            #[cfg(feature = "track_open_tags")]
            open_tags: vec![],
            #[cfg(feature = "track_open_tags")]
            closed_tags: vec![],
            #[cfg(feature = "parser_rules")]
            rule_stack: vec![],
            _custom_ty: PhantomData,
        }
    }
}

impl<'a, CustomTy> BBParser<'a, CustomTy>
where
    CustomTy: Clone,
{
    /// Returns all input text left to parse
    pub fn remaining(&self) -> &str {
        &self.input[self.loc..]
    }

    /// Returns all input text left to parse with the given offset from [BBParser::remaining][Self::remaining].
    pub fn remaining_after(&self, after: usize) -> &str {
        &self.input[(self.loc + after)..]
    }

    #[cfg(feature = "track_open_tags")]
    /// Returns all tags the parser believes to currently be open (i.e. no close block yet found)
    pub fn open_tags(&self) -> &[Token<'a, CustomTy>] {
        &self.open_tags
    }

    #[cfg(feature = "track_open_tags")]
    /// Returns all tags the parser believes to have been closed at some point (excluding standalone tags.)
    pub fn closed_tags(&self) -> &[Token<'a, CustomTy>] {
        &self.closed_tags
    }
}

impl<'a, CustomTy> Iterator for BBParser<'a, CustomTy>
where
    CustomTy: Clone,
{
    type Item = Token<'a, CustomTy>;

    fn next(&mut self) -> Option<Self::Item> {
        fn to_token_kind<'a, CustomTy>(tag: &'a str, args: &'a str) -> TokenKind<'a, CustomTy> {
            if tag.starts_with('/') {
                // End block.
                TokenKind::CloseBBTag(
                    BBTag {
                        tag: &tag["/".len()..],
                        args,
                    },
                    None,
                )
            } else if args.ends_with('/') {
                // Standalone block.
                TokenKind::StandaloneBBTag(BBTag {
                    tag,
                    args: &args[..(args.len() - "/".len())],
                })
            } else {
                TokenKind::OpenBBTag(BBTag { tag, args })
            }
        }

        fn to_token_kind_single<CustomTy>(tag: &str) -> TokenKind<'_, CustomTy> {
            if tag.starts_with('/') {
                // End block.
                TokenKind::CloseBBTag(
                    BBTag {
                        tag: &tag["/".len()..],
                        args: "",
                    },
                    None,
                )
            } else if tag.ends_with('/') {
                // Standalone block.
                TokenKind::StandaloneBBTag(BBTag {
                    tag: &tag[..(tag.len() - "/".len())],
                    args: "",
                })
            } else {
                TokenKind::OpenBBTag(BBTag { tag, args: "" })
            }
        }

        const TAG_OPENERS: &[char] = &['['];

        if self.loc >= self.input.len() {
            return None;
        }

        let first_char = self.remaining().chars().nth(0)?;

        let mut token = 'tk: {
            #[cfg(feature = "parser_rules")]
            {
                let action = self.rule_stack.last().map(|x| x.action());

                if let Some(rules::ParserRuleAction::CustomParser) = action {
                    let token = self.rule_stack.last_mut().unwrap().parse_custom(self.input);
                    self.loc += token.span.len();
                    break 'tk token;
                }
            }

            // If this block returns, then we failed to find any tag.
            'no_match: {
                if TAG_OPENERS.contains(&first_char) {
                    // We have a tag, figure out what it is.
                    let loc = first_char.len_utf8();
                    let rem_after = { &self.input[(self.loc + loc)..] };

                    let tag_end = rem_after.find(']');

                    // This would be better if we had try blocks in stable.
                    if tag_end.is_none() {
                        break 'no_match;
                    }

                    let tag_end = tag_end.unwrap();
                    // We live in a wonderful world where trim() does not allocate. Bless.
                    let tag_contents = rem_after[..tag_end].trim();

                    // Catch ""tags"" that contain another tag, and refuse them.
                    if tag_contents.matches(TAG_OPENERS).count() > 0 {
                        break 'no_match;
                    }

                    let span = &self.input[self.loc..(self.loc + tag_end + "[]".len())];
                    let old_loc = self.loc;
                    self.loc += span.len();

                    if let Some(arg_idx) = tag_contents.find(['=', ' ']) {
                        let (tag, args) = tag_contents.split_at(arg_idx);
                        // Inlined from Self::remaining() due to the borrowchecker not being able to see per-field borrows.

                        break 'tk Token::<'a, CustomTy> {
                            span,
                            start: old_loc,
                            kind: to_token_kind(tag, args),
                        };
                    } else {
                        break 'tk Token::<'a, CustomTy> {
                            span,
                            start: old_loc,
                            kind: to_token_kind_single(tag_contents),
                        };
                    }
                }
            }

            let segment_end = if !TAG_OPENERS.contains(&first_char) {
                self.remaining()
                    .match_indices(TAG_OPENERS)
                    .nth(0)
                    .map(|x| x.0)
                    .unwrap_or(self.remaining().len())
            } else {
                self.remaining_after("[".len())
                    .match_indices(TAG_OPENERS)
                    .nth(0)
                    .map(|x| x.0 + "[".len())
                    .unwrap_or(self.remaining().len())
            };

            let range = self.loc..(self.loc + segment_end);
            self.loc += range.len();
            break 'tk Token::<'a, CustomTy> {
                start: range.start,
                span: &self.input[range],
                kind: TokenKind::Text,
            };
        };

        #[cfg(feature = "parser_rules")]
        {
            let do_pop = if let Some(rule) = self.rule_stack.last() {
                rule.transform_token(&mut token)
            } else {
                false
            };

            if do_pop {
                self.rule_stack.pop();
            }

            let action = self.rule_stack.last().map(|x| x.action());

            if let Some(action) = action {
                match action {
                    rules::ParserRuleAction::NoParse => {
                        token.rewrite_as_text();
                    }
                    rules::ParserRuleAction::CustomParser => {}
                }
            }
        }

        #[cfg(feature = "track_open_tags")]
        {
            if let TokenKind::OpenBBTag(_) = token.kind {
                self.open_tags.push(token.clone());
            }

            if let TokenKind::CloseBBTag(BBTag { tag: removee, .. }, _) = token.kind {
                let to_remove: Option<usize> = 'blk: {
                    for (idx, tag) in self.open_tags.iter().enumerate().rev() {
                        if let TokenKind::OpenBBTag(ref t) = tag.kind {
                            if t.tag.eq_ignore_ascii_case(removee) {
                                break 'blk Some(idx);
                            } else if !self
                                .config
                                .feature_flags
                                .contains(ParserFeature::POP_UNORDERED)
                            {
                                break 'blk None;
                            }
                        } else {
                            unreachable!(
                                "Tag stack should never contain anything except open tags."
                            );
                        }
                    }

                    None
                };

                if let Some(to_remove) = to_remove {
                    // Might want to change the tags collection to be a linked list instead?
                    let tk = self.open_tags.remove(to_remove);
                    self.closed_tags.push(tk);
                    token.rewrite_with_opening_tag(self.closed_tags.len() - 1);
                    
                } else if self.config.feature_flags.contains(ParserFeature::UNMATCHED_CLOSE_AS_TEXT) {
                    token.rewrite_as_text();
                }
            }
        }

        Some(token)
    }
}

/// A parsed token, as returned by [BBParser::next].
#[derive(Clone)]
pub struct Token<'a, CustomTy>
where
    CustomTy: Clone,
{
    pub span: &'a str,
    pub start: usize,
    pub kind: TokenKind<'a, CustomTy>,
}

/// Properties of a Token like its arguments or kind.
impl<'a, CustomTy> Token<'a, CustomTy>
where
    CustomTy: Clone,
{
    /// The arguments for this tag, if any are present.
    /// # Remarks
    /// This will discard arguments that are purely whitespace as defined by [str::trim], and return None in those scenarios.
    pub fn args(&self) -> Option<&str> {
        let args = match self.kind {
            TokenKind::OpenBBTag(BBTag { args, .. }) => Some(args),
            TokenKind::CloseBBTag(BBTag { args, .. }, _) => Some(args),
            TokenKind::StandaloneBBTag(BBTag { args, .. }) => Some(args),
            _ => None,
        };

        match args {
            Some(args) if !args.trim().is_empty() => Some(args.trim()),
            _ => None,
        }
    }

    /// Whether or not this tag is an open tag of the given type.
    /// # Remarks
    /// This **ignores** the arguments of the tag, use [Token::is_open_argless] to ensure they're empty.
    pub fn is_open(&self, tag_name: &str) -> bool {
        if let TokenKind::OpenBBTag(BBTag { tag, .. }) = self.kind {
            tag.eq_ignore_ascii_case(tag_name)
        } else {
            false
        }
    }

    /// Whether or not this tag is an open tag of the given type, without arguments.
    pub fn is_open_argless(&self, tag_name: &str) -> bool {
        self.is_open(tag_name) && self.args().is_none()
    }

    /// Whether or not this tag is a close tag of the given type.
    /// # Remarks
    /// This **ignores** the arguments of the tag, use [Token::is_close_argless] to ensure they're empty.
    pub fn is_close(&self, tag_name: &str) -> bool {
        if let TokenKind::CloseBBTag(BBTag { tag, .. }, ..) = self.kind {
            tag.eq_ignore_ascii_case(tag_name)
        } else {
            false
        }
    }

    /// Whether or not this tag is a close tag of the given type, without arguments.
    pub fn is_close_argless(&self, tag_name: &str) -> bool {
        self.is_close(tag_name) && self.args().is_none()
    }

    /// Whether or not this tag is a standalone tag of the given type.
    /// # Remarks
    /// This **ignores** the arguments of the tag, use [Token::is_standalone_argless] to ensure they're empty.
    pub fn is_standalone(&self, tag_name: &str) -> bool {
        if let TokenKind::StandaloneBBTag(BBTag { tag, .. }) = self.kind {
            tag.eq_ignore_ascii_case(tag_name)
        } else {
            false
        }
    }

    /// Whether or not this tag is a standalone tag of the given type, without arguments.
    pub fn is_standalone_argless(&self, tag_name: &str) -> bool {
        self.is_standalone(tag_name) && self.args().is_none()
    }

    /// Whether or not this token is just plain text.
    pub fn is_text(&self) -> bool {
        matches!(self.kind, TokenKind::Text)
    }
}

/// Token "rewriters", which modify the token in-place.
impl<'a, CustomTy> Token<'a, CustomTy>
where
    CustomTy: Clone,
{
    /// Rewrite the token as text, in place, without altering any other properties.
    pub fn rewrite_as_text(&mut self) {
        self.kind = TokenKind::Text;
    }

    /// Mark the index of a close tag's opening tag within it, without altering anything else.
    /// # Panics
    /// Panics if the tag is not a closing tag.
    pub fn rewrite_with_opening_tag(&mut self, idx: usize) {
        let TokenKind::CloseBBTag(ref t, _) = self.kind else { unimplemented!("Can't set the opening tag index on anything except a closing tag!") };

        self.kind = TokenKind::CloseBBTag(t.clone(), Some(idx));
    }
}

impl<'a, CustomTy: core::fmt::Debug> core::fmt::Debug for Token<'a, CustomTy>
where
    CustomTy: Clone,
{
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Token")
            .field("span", &self.span)
            .field("start", &self.start)
            .field("kind", &self.kind)
            .finish()
    }
}

/// Simple struct representing the tag and (possibly empty) arguments of a bbcode tag.
#[derive(Debug, Clone)]
pub struct BBTag<'a> {
    /// A slice containing the tag.
    pub tag: &'a str,
    /// A slice containing the tag arguments.
    pub args: &'a str,
}

/// Represents the type of a token in the parsed data.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum TokenKind<'a, CustomTy = ()> {
    /// An opening tag in BBCode, ala `[tag]`.
    OpenBBTag(BBTag<'a>),
    /// A closing tag in BBCode, ala `[/tag]`
    CloseBBTag(BBTag<'a>, Option<usize>),
    /// A standalone (unpaired) tag in BBCode, ala `[tag/]`
    StandaloneBBTag(BBTag<'a>),
    /// Unformatted text.
    Text,
    /// A custom tag, emitted by a parser rule.
    /// # Remarks
    /// This could be removed entirely when parser rules aren't present, in the future.
    Custom(CustomTy),
}

#[cfg(feature = "parser_rules")]
/// Parser rules, which can be pushed into a [BBParser] mid-iteration to change how parsing behaves.
pub mod rules;

#[cfg(test)]
mod tests;
