use std::marker::PhantomData;

use crate::{BBParser, Token};

pub enum ParserRuleAction {
    /// Implement a fully custom parse action, allowing the user to emit [`TokenKind::Custom`][`super::TokenKind::Custom`]s.
    /// See [ParserRule::parse_custom] for details on how this works.
    CustomParser,
    /// Disable parsing within the rule's domain, de-tokenizing any parsed tokens back into their string form until the parser "releases".
    NoParse,
}

pub trait ParserRule<'a, CustomTy = ()>
where
    CustomTy: Clone,
{
    const ACTION: ParserRuleAction;

    fn check_should_release<'b>(&self, next: &Token<'b, CustomTy>) -> bool;

    /// Provides a mechanism for
    fn parse_custom<'b: 'a>(&'a mut self, _parser: &'b BBParser<'b>) -> Option<CustomTy> {
        unimplemented!("Parse custom triggered ")
    }
}

pub(super) trait ParserRuleInner<'a, CustomTy = ()>
where
    CustomTy: Clone,
{
    fn action(&self) -> ParserRuleAction;

    fn check_should_release<'b>(&self, next: &Token<'b, CustomTy>) -> bool;

    fn parse_custom<'b: 'a>(&'a mut self, _parser: &'b BBParser<'b>) -> Option<CustomTy>;
}

pub(super) struct ParserRuleImpl<'a, Rule, CustomTy>
where
    Rule: ParserRule<'a, CustomTy> + ?Sized,
    CustomTy: Clone,
{
    pub _customty: PhantomData<CustomTy>,
    pub _lifetime: PhantomData<&'a ()>,
    pub rule: Rule,
}

impl<'a, Rule, CustomTy> ParserRuleInner<'a, CustomTy> for ParserRuleImpl<'a, Rule, CustomTy>
where
    Rule: ParserRule<'a, CustomTy>,
    CustomTy: Clone,
{
    fn action(&self) -> ParserRuleAction {
        Rule::ACTION
    }

    fn check_should_release<'b>(&self, next: &Token<'b, CustomTy>) -> bool {
        self.rule.check_should_release(next)
    }

    fn parse_custom<'b: 'a>(&'a mut self, parser: &'b BBParser<'b>) -> Option<CustomTy> {
        self.rule.parse_custom(parser)
    }
}

pub mod builtin {
    use std::marker::PhantomData;

    use crate::{parser::BBTag, Token, TokenKind};

    use super::{ParserRule, ParserRuleAction};

    pub struct NoParseRule<'a, CustomTy = ()> {
        _custom_ty: PhantomData<CustomTy>,
        tag_name: &'a str,
    }

    impl<'a, CustomTy> NoParseRule<'a, CustomTy> {
        pub fn new(tag_name: &'a str) -> Self {
            Self {
                _custom_ty: PhantomData,
                tag_name,
            }
        }
    }

    impl<'a, 'rule_life: 'a, CustomTy> ParserRule<'a, CustomTy> for NoParseRule<'rule_life, CustomTy>
    where
        CustomTy: Clone,
    {
        const ACTION: ParserRuleAction = ParserRuleAction::NoParse;

        fn check_should_release<'b>(&self, next: &Token<'b, CustomTy>) -> bool {
            if let TokenKind::CloseBBTag(BBTag { tag, .. }) = next.kind {
                tag.eq_ignore_ascii_case(self.tag_name)
            } else {
                false
            }
        }
    }
}
