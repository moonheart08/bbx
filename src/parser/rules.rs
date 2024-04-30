use core::marker::PhantomData;

use crate::{BBParser, Token};

/// Represents an action a parser rule can take every [BBParser::next] call.
pub enum ParserRuleAction {
    /// Implement a fully custom parse action, allowing the user to emit [`TokenKind::Custom`][`super::TokenKind::Custom`]s.
    /// See [ParserRule::parse_custom] for details on how this works.
    CustomParser,
    /// Disable parsing within the rule's domain, de-tokenizing any parsed tokens back into their string form until the parser "releases".
    NoParse,
}

/// Provides the common API for parser rules, 
pub trait ParserRule<'a, CustomTy = ()>
where
    CustomTy: Clone,
{
    const ACTION: ParserRuleAction;

    /// Check if this rule should be released (removed) from the current parser, given the next token to be returned.
    fn check_should_release(&self, next: &Token<'_, CustomTy>) -> bool;

    //TODO: Problem for tomorrow's me, this API doesn't actually work, custom parsers can never release and cannot control how much they parse.
    /// Provides a mechanism for custom parsing logic, should [ParserRule::ACTION] be [ParserRuleAction::CustomParser].
    /// Will not be called otherwise.
    fn parse_custom<'b: 'a>(&mut self, _parser: &'b BBParser<'b>) -> Option<CustomTy> {
        unimplemented!("Parse custom triggered ")
    }
}

pub(super) trait ParserRuleInner<'a, CustomTy = ()>
where
    CustomTy: Clone,
{
    fn action(&self) -> ParserRuleAction;

    fn check_should_release(&self, next: &Token<'_, CustomTy>) -> bool;

    fn parse_custom<'b: 'a>(&mut self, _parser: &'b BBParser<'b>) -> Option<CustomTy>;
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

    fn check_should_release(&self, next: &Token<'_, CustomTy>) -> bool {
        self.rule.check_should_release(next)
    }

    fn parse_custom<'b: 'a>(&mut self, parser: &'b BBParser<'b>) -> Option<CustomTy> {
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

        fn check_should_release(&self, next: &Token<'_, CustomTy>) -> bool {
            if let TokenKind::CloseBBTag(BBTag { tag, .. }, ..) = next.kind {
                tag.eq_ignore_ascii_case(self.tag_name)
            } else {
                false
            }
        }
    }
}

impl<'a, CustomTy> BBParser<'a, CustomTy>
where
    CustomTy: Clone,
{
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
}
