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

/// Provides the common API for parser rules, allowing the programmer to modify parsing behavior.
pub trait ParserRule<'a, CustomTy = ()>
where
    CustomTy: Clone,
{
    const ACTION: ParserRuleAction;

    /// Called whenever a new token has been produced, allowing the rule to transform a token. Upon returning false, the parser rule will be removed from the rule stack.
    /// # Remarks
    /// [transform_token][ParserRule::transform_token] is always called **before** open/close tag tracking, as such the current set of open tags will not contain the tag given,
    /// and the transformer can emit open/close tags and expect them to be tracked correctly.
    fn transform_token(&self, token: &mut Token<'_, CustomTy>) -> bool;

    /// Provides a mechanism for custom parsing logic, should [ParserRule::ACTION] be [ParserRuleAction::CustomParser].
    /// Will not be called otherwise.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn parse_custom<'b: 'a>(&mut self, _parser: &'b str) -> Token<'a, CustomTy> {
        unimplemented!("Parse custom triggered, but not implemented.")
    }
}

/// Internal wrapper over parse rules to make them object safe, this is the trait half of the deal.
pub(super) trait ParserRuleInner<'a, CustomTy = ()>
where
    CustomTy: Clone,
{
    fn action(&self) -> ParserRuleAction;

    fn transform_token(&self, next: &mut Token<'_, CustomTy>) -> bool;

    fn parse_custom<'b: 'a>(&mut self, _parser: &'b str) -> Token<'a, CustomTy>;
}

/// Internal wrapper over parse rules to make them object safe, this is the struct containing the user provided rule.
pub(super) struct ParserRuleImpl<'a, Rule, CustomTy>
where
    Rule: ParserRule<'a, CustomTy> + ?Sized,
    CustomTy: Clone,
{
    pub _customty: PhantomData<CustomTy>,
    pub _lifetime: PhantomData<&'a ()>,
    pub rule: Rule,
}

//SAFETY: This is safe, we only implement Send when Rule is send, and CustomTy is never stored anywhere within ParserRuleImpl itself.
//SAFETY: If there was a way to have PhantomData always implement Send+Sync (as we never store CustomTy ourselves, and if the Rule does their type reflects that), then this would be unnecessary.
unsafe impl<'a, Rule, CustomTy> Send for ParserRuleImpl<'a, Rule, CustomTy>
where
    Rule: ParserRule<'a, CustomTy> + ?Sized + Send,
    CustomTy: Clone,
{
}

impl<'a, Rule, CustomTy> ParserRuleInner<'a, CustomTy> for ParserRuleImpl<'a, Rule, CustomTy>
where
    Rule: ParserRule<'a, CustomTy>,
    CustomTy: Clone,
{
    fn action(&self) -> ParserRuleAction {
        Rule::ACTION
    }

    fn transform_token(&self, next: &mut Token<'_, CustomTy>) -> bool {
        self.rule.transform_token(next)
    }

    fn parse_custom<'b: 'a>(&mut self, parser: &'b str) -> Token<'a, CustomTy> {
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

        fn transform_token(&self, next: &mut Token<'_, CustomTy>) -> bool {
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
    CustomTy: Clone + 'a,
{
    pub fn push_rule<Rule>(&mut self, rule: Rule)
    where
        Rule: ParserRule<'a, CustomTy> + Send + 'a,
    {
        self.rule_stack
            .push(Box::new(ParserRuleImpl::<'a, Rule, CustomTy> {
                _customty: PhantomData,
                _lifetime: PhantomData,
                rule,
            }))
    }
}
