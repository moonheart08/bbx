use std::{marker::PhantomData, vec};

use bitflags::bitflags;

use self::rules::{ParserRule, ParserRuleImpl};

#[derive(Default)]
pub struct ParserConfig {}

bitflags! {
    /// Represents a set of flags.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct ParserFeature: u32 {
        /// Enable parse errors, disabled by default for maximum conformance.
        const ERRORS = 1 << 1;
        /*
        /// Enable pseudohtml support, which allows the parser to match on `<>` bracketed tags.
        const PSEUDOHTML = 1 << 2;
        */

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
        const TAG_OPENERS: &[char] = &['['];

        if self.loc >= self.input.len() {
            return None;
        }

        let first_char = self.remaining().chars().nth(0)?;

        let mut token = 'tk: {
            if TAG_OPENERS.contains(&first_char) {
                // We have a tag, figure out what it is.
                self.loc += first_char.len_utf8();
            } else {
                // Just text.
                let segment_end = self
                    .remaining()
                    .match_indices(TAG_OPENERS)
                    .nth(0)
                    .map(|x| x.0)
                    .unwrap_or(self.input.len());
                let range = self.loc..segment_end;
                self.loc += range.len();
                break 'tk Token::<'a, CustomTy> {
                    start: range.start,
                    span: &self.input[range],
                    kind: TokenKind::Text,
                };
            }
            unimplemented!()
        };

        #[cfg(feature = "track_open_tags")]
        {
            if let TokenKind::OpenBBTag(_) = token.kind {
                self.tags.push(token.clone());
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

#[derive(Clone)]
pub struct Token<'a, CustomTy>
where
    CustomTy: Clone,
{
    pub span: &'a str,
    pub start: usize,
    pub kind: TokenKind<'a, CustomTy>,
}

#[derive(Clone)]
pub struct BBTag<'a> {
    pub tag: &'a str,
    pub args: &'a str,
}

#[derive(Clone)]
pub enum TokenKind<'a, CustomTy = ()> {
    OpenBBTag(BBTag<'a>),
    CloseBBTag(BBTag<'a>),
    StandaloneBBTag(BBTag<'a>),
    Text,
    #[cfg(feature = "parser_rules")]
    Custom(CustomTy),
}

#[cfg(test)]
mod tests {
    use crate::{BBParser, TokenKind};

    const LOREM_IPSUM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. In lorem quam, fermentum id porttitor ac, iaculis eu arcu. Aliquam vulputate tempus felis consequat elementum. Cras auctor nunc a cursus lobortis. Fusce venenatis quam nec eleifend porta. Nulla velit diam, maximus sed lobortis imperdiet, hendrerit id elit. Integer congue congue porttitor. Curabitur at erat urna. Morbi iaculis felis eu est cursus, eu imperdiet nibh consectetur. Proin nisi metus, blandit non placerat hendrerit, facilisis id metus. Aenean fringilla, justo id venenatis rutrum, erat ex vehicula sapien, convallis aliquam augue turpis venenatis risus. In nulla lacus, auctor vitae sapien vel, tristique venenatis mi. Sed iaculis iaculis aliquet.";

    #[test]
    pub fn just_text() {
        let mut parser: BBParser<'static, ()> = BBParser::new(LOREM_IPSUM);
        assert!(parser.all(|x| matches!(x.kind, TokenKind::Text)))
    }
}
