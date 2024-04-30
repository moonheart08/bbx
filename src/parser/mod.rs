use std::{marker::PhantomData, vec};

use bitflags::{bitflags, Flags};

use self::rules::{ParserRule, ParserRuleImpl};

#[derive(Default)]
pub struct ParserConfig {
    pub feature_flags: ParserFeature,
}

bitflags! {
    /// Represents a set of flags.
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ParserFeature: u32 {
        /// Allow tags to be removed out of order.
        const POP_UNORDERED = 1 << 1;

        /// All current and future feature flags.
        /// Prefer using one of the versioned versions of this mask (i.e. V1) if you have stability guarantees.
        const ALL = u32::MAX;
    }
}

#[cfg(feature = "parser_rules")]
pub mod rules;

#[doc(alias = "parser")]
pub struct BBParser<'a, CustomTy = ()>
where
    CustomTy: Clone,
{
    input: &'a str,
    config: ParserConfig,
    loc: usize,
    #[cfg(feature = "track_open_tags")]
    tags: Vec<Token<'a, CustomTy>>,
    #[cfg(feature = "parser_rules")]
    rule_stack: Vec<Box<dyn rules::ParserRuleInner<'a, CustomTy> + 'a>>,
    _custom_ty: PhantomData<CustomTy>,
}

impl<'a, CustomTy> BBParser<'a, CustomTy>
where
    CustomTy: Clone,
{
    /// Returns all input text left to parse
    pub fn remaining(&self) -> &str {
        &self.input[self.loc..]
    }

    pub fn remaining_after(&self, after: usize) -> &str {
        &self.input[(self.loc + after)..]
    }

    pub fn push_rule<Rule>(&'a mut self, rule: Rule)
    where
        Rule: ParserRule<'a, CustomTy> + 'a,
    {
        self.rule_stack
            .push(Box::new(ParserRuleImpl::<'a, Rule, CustomTy> {
                _customty: PhantomData,
                _lifetime: PhantomData,
                rule,
            }))
    }

    pub fn new(input: &'a str) -> BBParser<'a, CustomTy> {
        Self {
            input,
            config: Default::default(),
            loc: 0,
            #[cfg(feature = "track_open_tags")]
            tags: vec![],
            #[cfg(feature = "parser_rules")]
            rule_stack: vec![],
            _custom_ty: PhantomData,
        }
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
                TokenKind::CloseBBTag(BBTag { tag: &tag["/".len()..], args }, None)
            } else if args.ends_with('/') {
                // Standalone block.
                TokenKind::StandaloneBBTag(BBTag { tag, args: &args[..(args.len() - "/".len())] })
            } else {
                TokenKind::OpenBBTag(BBTag { tag, args })
            }
        }

        fn to_token_kind_single<'a, CustomTy>(tag: &'a str) -> TokenKind<'a, CustomTy> {
            
            if tag.starts_with('/') {
                // End block.
                TokenKind::CloseBBTag(BBTag { tag: &tag["/".len()..], args: ""}, None)
            } else if tag.ends_with('/') {
                // Standalone block.
                TokenKind::StandaloneBBTag(BBTag { tag: &tag[..(tag.len() - "/".len())], args: "" })
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
            // If this block returns, then we failed to find any tag.
            'no_match: {
                if TAG_OPENERS.contains(&first_char) {
                    // We have a tag, figure out what it is.
                    let loc = first_char.len_utf8();
                    let rem_after = {
                        &self.input[(self.loc + loc)..]
                    };

                    let tag_end = rem_after.find(']');

                    // This would be better if we had try blocks in stable.
                    if tag_end.is_none() {
                        break 'no_match;
                    }

                    let tag_end = tag_end.unwrap();
                    // We live in a wonderful world where trim() does not allocate. Bless.
                    let tag_contents = rem_after[..tag_end].trim();
                    
                    let span = &{
                        &self.input[self.loc..]
                    }[..(tag_contents.len() + "[]".len())];
                    let old_loc = self.loc;
                    self.loc += span.len();

                    if let Some(arg_idx) = tag_contents.find(['=', ' ']) {
                        let (tag, args) = tag_contents.split_at(arg_idx);
                        // Inlined from Self::remaining() due to the borrowchecker not being able to see per-field borrows.
                        
                        break 'tk Token::<'a, CustomTy> {
                            span: span,
                            start: old_loc,
                            kind: to_token_kind(tag, args)
                        }
                    } else {
                        break 'tk Token::<'a, CustomTy> {
                            span: span,
                            start: old_loc,
                            kind: to_token_kind_single(tag_contents)
                        }
                    }
                }
            }

            let segment_end = self
                .remaining()
                .match_indices(TAG_OPENERS)
                .nth(0)
                .map(|x| x.0)
                .unwrap_or(self.remaining().len());

            let range = self.loc..(self.loc+segment_end);
            self.loc += range.len();
            break 'tk Token::<'a, CustomTy> {
                start: range.start,
                span: &self.input[range],
                kind: TokenKind::Text,
            };
        };

        #[cfg(feature = "track_open_tags")]
        {
            if let TokenKind::OpenBBTag(_) = token.kind {
                self.tags.push(token.clone());
            }

            if let TokenKind::CloseBBTag(BBTag { tag: removee, .. }, _) = token.kind {
                let to_remove: Option<usize> = 'blk: { 
                    for (idx, tag) in self.tags.iter().enumerate().rev() {
                        if let TokenKind::OpenBBTag(ref t) = tag.kind {
                            if t.tag.eq_ignore_ascii_case(removee) {
                                break 'blk Some(idx);
                            } else if !self.config.feature_flags.contains(ParserFeature::POP_UNORDERED) {
                                break 'blk None;
                            }
                        } else {
                            unreachable!("Tag stack should never contain anything except open tags.");
                        }
                    }

                    None
                };

                if let Some(to_remove) = to_remove {
                    // Might want to change the tags collection to be a linked list instead?
                    self.tags.remove(to_remove);
                }
            }
        }

        #[cfg(feature = "parser_rules")]
        {
            let do_pop = if let Some(rule) = self.rule_stack.last() {
                rule.check_should_release(&token)
            } else {
                false
            };

            if do_pop {
                self.rule_stack.pop();
            }

            let action = self.rule_stack.last().map(|x| x.action());

            if let Some(action) = action {
                match action {
                    rules::ParserRuleAction::CustomParser => {
                        todo!()
                    }
                    rules::ParserRuleAction::NoParse => {
                        token = Token::<'a, CustomTy> {
                            kind: TokenKind::Text,
                            ..token
                        };
                    }
                }
            }
        }

        Some(token)
    }
}

#[derive(Debug, Clone)]
pub struct Token<'a, CustomTy>
where
    CustomTy: Clone,
{
    pub span: &'a str,
    pub start: usize,
    pub kind: TokenKind<'a, CustomTy>,
}

#[derive(Debug, Clone)]
pub struct BBTag<'a> {
    pub tag: &'a str,
    pub args: &'a str,
}

#[derive(Debug, Clone)]
pub enum TokenKind<'a, CustomTy = ()> {
    OpenBBTag(BBTag<'a>),
    CloseBBTag(BBTag<'a>, Option<usize>),
    StandaloneBBTag(BBTag<'a>),
    Text,
    #[cfg(feature = "parser_rules")]
    Custom(CustomTy),
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::{BBParser, Token, TokenKind};

    const LOREM_IPSUM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. In lorem quam, fermentum id porttitor ac, iaculis eu arcu. Aliquam vulputate tempus felis consequat elementum. Cras auctor nunc a cursus lobortis. Fusce venenatis quam nec eleifend porta. Nulla velit diam, maximus sed lobortis imperdiet, hendrerit id elit. Integer congue congue porttitor. Curabitur at erat urna. Morbi iaculis felis eu est cursus, eu imperdiet nibh consectetur. Proin nisi metus, blandit non placerat hendrerit, facilisis id metus. Aenean fringilla, justo id venenatis rutrum, erat ex vehicula sapien, convallis aliquam augue turpis venenatis risus. In nulla lacus, auctor vitae sapien vel, tristique venenatis mi. Sed iaculis iaculis aliquet.";

    const SIMPLE: &str = "[bold]This is a test![/bold] and it's very cool.";

    #[test]
    pub fn just_text() {
        let mut parser: BBParser<'static, ()> = BBParser::new(LOREM_IPSUM);
        assert!(parser.all(|x| matches!(x.kind, TokenKind::Text)))
    }

    #[test]
    pub fn simple_tags() {
        let mut parser: BBParser<'static, ()> = BBParser::new(SIMPLE);
        assert_matches!(
            parser.next(),
            Some(Token {
                kind: TokenKind::OpenBBTag(_),
                ..
            })
        );

        assert_matches!(
            parser.next(),
            Some(Token {
                kind: TokenKind::Text,
                ..
            })
        );

        assert_matches!(
            parser.next(),
            Some(Token {
                kind: TokenKind::CloseBBTag(..),
                ..
            })
        );

        
        assert_matches!(
            parser.next(),
            Some(Token {
                kind: TokenKind::Text,
                ..
            })
        );

        assert_matches!(parser.next(), None);
    }
}
