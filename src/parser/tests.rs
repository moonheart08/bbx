use crate::{parser::ParserFeature, BBParser, Token, TokenKind};

const LOREM_IPSUM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. In lorem quam, fermentum id porttitor ac, iaculis eu arcu. Aliquam vulputate tempus felis consequat elementum. Cras auctor nunc a cursus lobortis. Fusce venenatis quam nec eleifend porta. Nulla velit diam, maximus sed lobortis imperdiet, hendrerit id elit. Integer congue congue porttitor. Curabitur at erat urna. Morbi iaculis felis eu est cursus, eu imperdiet nibh consectetur. Proin nisi metus, blandit non placerat hendrerit, facilisis id metus. Aenean fringilla, justo id venenatis rutrum, erat ex vehicula sapien, convallis aliquam augue turpis venenatis risus. In nulla lacus, auctor vitae sapien vel, tristique venenatis mi. Sed iaculis iaculis aliquet.";

#[test]
pub fn just_text() {
    let mut parser = BBParser::new(LOREM_IPSUM);
    let tok = parser.next().unwrap();
    assert!(tok.is_text());
    assert!(tok.args().is_none());
    assert!(parser.next().is_none())
}

const SIMPLE: &str = "[bold]This is a test![/bold] and it's very cool.";

#[test]
pub fn simple_tags() {
    let mut parser = BBParser::new(SIMPLE);
    let bold_tag = parser.next().unwrap();
    assert!(bold_tag.is_open("bold"));
    assert!(bold_tag.is_open_argless("bold"));
    assert!(!bold_tag.is_close("bold"));
    assert!(!bold_tag.is_close_argless("bold"));
    assert!(!bold_tag.is_standalone("bold"));
    assert!(!bold_tag.is_standalone_argless("bold"));

    assert!(matches!(
        parser.next(),
        Some(Token {
            kind: TokenKind::Text,
            ..
        })
    ));

    assert!(matches!(
        parser.next(),
        Some(Token {
            kind: TokenKind::CloseBBTag(..),
            ..
        })
    ));

    assert!(matches!(
        parser.next(),
        Some(Token {
            kind: TokenKind::Text,
            ..
        })
    ));

    assert!(parser.next().is_none());
}

const NO_PARSE_RULE: &str = "[ bar ] [ noparse ]foo [/bar] [baz] asdfasd [/noparse]";

#[cfg(feature = "track_open_tags")]
#[test]
pub fn no_parse_rule() {
    use crate::rules;

    let mut parser = BBParser::new(NO_PARSE_RULE);

    // Can't use a for loop here, unfortunately, as it keeps a permamnent mutable ref to parser.
    while let Some(tk) = parser.next() {
        if tk.is_open("noparse") {
            parser.push_rule(rules::builtin::NoParseRule::new("noparse"));
        }
    }

    let open_tags = parser.open_tags();
    assert!(open_tags.iter().any(|x| x.is_open("bar")));
}

const NO_TAG_BLEED: &str = "[bar ]foo";

// Issue found on commit 10570230da3f065920408df5c05063790e746ae1 where tags didn't properly capture their ending bracket if there was whitespace.
#[test]
pub fn no_tag_bleed() {
    let mut parser = BBParser::new(NO_TAG_BLEED);
    let bar = parser.next().unwrap();
    assert!(bar.span.contains(']'));
    let text = parser.next().unwrap();
    assert!(!text.span.contains(']'))
}

const TAG_KINDS: &str = "[open_argless][open args][open=args][/close_argless][/close args][/close=args][standalone_argless/][standalone args/][standalone=args/]";

#[test]
pub fn tag_kinds() {
    let parser = BBParser::new(TAG_KINDS);
    tag_kinds_inner(parser);
    let parser = BBParser::with_config(
        TAG_KINDS,
        crate::ParserConfig {
            feature_flags: ParserFeature::POP_UNORDERED,
        },
    );
    tag_kinds_inner(parser);
}

fn tag_kinds_inner(mut parser: BBParser<'static>) {
    // [open_argless]
    let tag = parser.next().unwrap();
    assert!(tag.is_open_argless("open_argless"));
    // [open args]
    let tag = parser.next().unwrap();
    assert!(!tag.is_open_argless("open"));
    assert!(tag.is_open("open"));
    // [open=args]
    let tag = parser.next().unwrap();
    assert!(!tag.is_open_argless("open"));
    assert!(tag.is_open("open"));
    // [close_argless]
    let tag = parser.next().unwrap();
    assert!(tag.is_close_argless("close_argless"));
    // [close args]
    let tag = parser.next().unwrap();
    assert!(!tag.is_close_argless("close"));
    assert!(tag.is_close("close"));
    // [close=args]
    let tag = parser.next().unwrap();
    assert!(!tag.is_close_argless("close"));
    assert!(tag.is_close("close"));
    // [standalone_argless]
    let tag = parser.next().unwrap();
    assert!(tag.is_standalone_argless("standalone_argless"));
    // [standalone args]
    let tag = parser.next().unwrap();
    assert!(!tag.is_standalone_argless("standalone"));
    assert!(tag.is_standalone("standalone"));
    // [standalone=args]
    let tag = parser.next().unwrap();
    assert!(!tag.is_standalone_argless("standalone"));
    assert!(tag.is_standalone("standalone"));
}

const UNCLOSED_TAG: &str = "[not_a_tag=real ";

#[test]
pub fn unclosed_tag() {
    let mut parser = BBParser::new(UNCLOSED_TAG);

    assert!(parser.next().unwrap().is_text());
    assert!(parser.next().is_none());
}
