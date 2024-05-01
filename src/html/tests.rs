use crate::{html::builtins, BBParser};

use super::{HtmlSerializer, SimpleHtmlWriter};

const SIMPLE_ESCAPED: &str = "[b]Foo<b>bar[/b] baz.";

#[test]
pub fn simple_escaped() {
    let parser = BBParser::new(SIMPLE_ESCAPED);
    let mut serializer =
        HtmlSerializer::<SimpleHtmlWriter>::with_tags(builtins::all_core_v1_tags());

    assert_eq!(serializer.serialize(parser), "<b>Foo&lt;b&gt;bar</b> baz.");
}


const SIMPLE: &str = "[title]This is a test![/title]";

#[test]
pub fn simple() {
    let parser = BBParser::new(SIMPLE);
    let mut serializer =
        HtmlSerializer::<SimpleHtmlWriter>::with_tags(builtins::all_core_v1_tags());

    assert_eq!(serializer.serialize(parser), "<h1>This is a test!</h1>");
}
