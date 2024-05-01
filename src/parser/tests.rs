use crate::{BBParser, Token, TokenKind};

const LOREM_IPSUM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. In lorem quam, fermentum id porttitor ac, iaculis eu arcu. Aliquam vulputate tempus felis consequat elementum. Cras auctor nunc a cursus lobortis. Fusce venenatis quam nec eleifend porta. Nulla velit diam, maximus sed lobortis imperdiet, hendrerit id elit. Integer congue congue porttitor. Curabitur at erat urna. Morbi iaculis felis eu est cursus, eu imperdiet nibh consectetur. Proin nisi metus, blandit non placerat hendrerit, facilisis id metus. Aenean fringilla, justo id venenatis rutrum, erat ex vehicula sapien, convallis aliquam augue turpis venenatis risus. In nulla lacus, auctor vitae sapien vel, tristique venenatis mi. Sed iaculis iaculis aliquet.";

#[test]
pub fn just_text() {
    let mut parser = BBParser::new(LOREM_IPSUM);
    assert!(parser.all(|x| matches!(x.kind, TokenKind::Text)))
}

const SIMPLE: &str = "[bold]This is a test![/bold] and it's very cool.";

#[test]
pub fn simple_tags() {
    let mut parser = BBParser::new(SIMPLE);
    assert!(matches!(
        parser.next(),
        Some(Token {
            kind: TokenKind::OpenBBTag(_),
            ..
        }))
    );

    assert!(matches!(
        parser.next(),
        Some(Token {
            kind: TokenKind::Text,
            ..
        }))
    );

    assert!(matches!(
        parser.next(),
        Some(Token {
            kind: TokenKind::CloseBBTag(..),
            ..
        }))
    );

    assert!(matches!(
        parser.next(),
        Some(Token {
            kind: TokenKind::Text,
            ..
        }))
    );

    assert!(matches!(parser.next(), None));
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
        println!("{:?}", tk);
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
    println!("{:?}", bar);
    assert!(bar.span.contains("]"));
    let text = parser.next().unwrap();
    println!("{:?}", text);
    assert!(!text.span.contains("]"))
}