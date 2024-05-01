//! Built-in implementations of common (i.e. used by many implementations, subjectively) BBCode tags.
use core::marker::PhantomData;

use super::HtmlTagWriter;

impl<T: SimpleHtmlTagWriter<CustomTy>, CustomTy> HtmlTagWriter<CustomTy> for T
    where CustomTy: Clone
{
    fn match_tag(&self, tag: &str) -> bool {
        Self::TAGS.iter().any(|x| x.eq_ignore_ascii_case(tag))
    }

    fn open_tag<'a>(
        &self,
        tk_writer: &dyn super::HtmlTokenWriter<CustomTy>,
        token: &crate::Token<'a, CustomTy>,
        out: &mut String,
    ) {
        // TODO: This could use let expressions when those are stable late-bound.
        if Self::TAGS.iter().any(|x| token.is_open_argless(x)) && Self::HTML_OPEN.is_some() {
            out.push_str(Self::HTML_OPEN.unwrap())
        } else {
            tk_writer.write_token(token, out)
        }
    }

    fn close_tag<'a>(
        &self,
        tk_writer: &dyn super::HtmlTokenWriter<CustomTy>,
        _: &crate::Token<'a, CustomTy>,
        close_token: &crate::Token<'a, CustomTy>,
        out: &mut String,
    ) {
        if Self::TAGS.iter().any(|x| close_token.is_close_argless(x)) && Self::HTML_CLOSE.is_some() {
            out.push_str(Self::HTML_CLOSE.unwrap())
        } else {
            tk_writer.write_token(close_token, out)
        }
    }

    fn standalone_tag<'a>(
        &self,
        tk_writer: &dyn super::HtmlTokenWriter<CustomTy>,
        token: &crate::Token<'a, CustomTy>,
        out: &mut String,
    ) {
        if Self::TAGS.iter().any(|x| token.is_standalone_argless(x)) && Self::HTML_STANDALONE.is_some() {
            out.push_str(Self::HTML_STANDALONE.unwrap())
        } else {
            tk_writer.write_token(token, out)
        }
    }
}

trait SimpleHtmlTagWriter<CustomTy>
    where
        CustomTy: Clone,
{
    const TAGS: &'static [&'static str];

    const HTML_OPEN: Option<&'static str>;

    const HTML_CLOSE: Option<&'static str>;

    const HTML_STANDALONE: Option<&'static str>;
}

macro_rules! simple_tag {
    ($doc:expr, $name:ident, $tags:expr, $open:expr, $close:expr) => {
        #[derive(Copy, Clone, Debug, Default)]
        #[doc = $doc]
        #[doc = "<br/>"]
        #[doc = "This matches the following BBCode tags: `"]
        #[doc = stringify!($tags)]
        #[doc = "`"]
        #[doc = "# Exact output"]
        #[doc = "This tag converts exactly to"]
        #[doc = "```html"]
        #[doc = $open]
        #[doc = " contents"]
        #[doc = $close]
        #[doc = "```"]
        pub struct $name<CustomTy = ()> {
            _custom_ty: PhantomData<CustomTy>,
        }
    
        impl<CustomTy> SimpleHtmlTagWriter<CustomTy> for $name<CustomTy>
        where
            CustomTy: Clone,
        {
            const TAGS: &'static [&'static str] = &$tags;
            const HTML_CLOSE: Option<&'static str> = Some($close);
            const HTML_OPEN: Option<&'static str> = Some($open);
            const HTML_STANDALONE: Option<&'static str> = None;
        }
    };
}

macro_rules! simple_standalone_tag {
    ($doc:expr, $name:ident, $tags:expr, $standalone:expr) => {
        #[derive(Copy, Clone, Debug, Default)]
        #[doc = $doc]
        #[doc = "<br/>"]
        #[doc = "This matches the following BBCode tags: `"]
        #[doc = stringify!($tags)]
        #[doc = "`"]
        #[doc = "# Exact output"]
        #[doc = "This tag converts exactly to"]
        #[doc = "```html"]
        #[doc = $standalone]
        #[doc = "```"]
        pub struct $name<CustomTy = ()> {
            _custom_ty: PhantomData<CustomTy>,
        }
    
        impl<CustomTy> SimpleHtmlTagWriter<CustomTy> for $name<CustomTy>
        where
            CustomTy: Clone,
        {
            const TAGS: &'static [&'static str] = &$tags;
            const HTML_CLOSE: Option<&'static str> = None;
            const HTML_OPEN: Option<&'static str> = None;
            const HTML_STANDALONE: Option<&'static str> = Some($standalone);
        }
    };
}

// "Safe" tags, as per the definition in all_core_v1_tags.
simple_tag!{
    "A bold tag with no arguments, which converts directly to HTML5 `<b>`.",
    BoldTag, ["b", "bold"], "<b>", "</b>"
}
simple_tag!{
    "An italic tag with no arguments, which converts directly to HTML5 `<i>`.",
    ItalicTag, ["i", "italic"], "<i>", "</i>"
}
simple_tag!{
    "An underline tag no arguments, which converts directly to HTML5 `<u>`.",
    UnderlineTag, ["u", "underline", "under"], "<u>", "</u>"
}
simple_standalone_tag!{
    "A linebreak tag with no arguments, which converts directly into HTML5 `<br/>`.",
    LinebreakTag, ["br"], "<br/>"
}
simple_tag!{
    "A block quote tag with no arguments, which converts directly to HTML5 `<blockquote>`.",
    BlockQuoteTag, ["quote", "blockquote"], "<blockquote>", "</blockquote>"
}
simple_tag!{
    "Inline quote tag with no arguments, which converts directly to HTML5 `<q>`.",
    QuoteTag, ["q"], "<q>", "</q>"
}
simple_tag!{
    "Subscript tag with no arguments, which converts directly to HTML5 `<sub>`.",
    SubscriptTag, ["sub", "subscript", "small"], "<sub>", "</sub>"
}
simple_tag!{
    "Superscript tag with no arguments, which converts directly to HTML5 `<sup>`.",
    SuperscriptTag, ["sup", "super", "superscript"], "<sup>", "</sup>"
}
simple_tag!{
    "Header (tier 1) tag with no arguments, which converts directly to HTML5 `<h1>`.",
    Header1Tag, ["h1", "title"], "<h1>", "</h1>"
}
simple_tag!{
    "Header (tier 2) tag with no arguments, which converts directly to HTML5 `<h2>`.",
    Header2Tag, ["h2", "topic"], "<h2>", "</h2>"
}
simple_tag!{
    "Header (tier 3) tag with no arguments, which converts directly to HTML5 `<h3>`.",
    Header3Tag, ["h3", "subtopic"], "<h3>", "</h3>"
}
simple_tag!{
    "Header (tier 4) tag with no arguments, which converts directly to HTML5 `<h4>`.",
    Header4Tag, ["h4"], "<h4>", "</h4>"
}
simple_tag!{
    "Header (tier 5) tag with no arguments, which converts directly to HTML5 `<h5>`.",
    Header5Tag, ["h5"], "<h5>", "</h5>"
}
simple_tag!{
    "Header (tier 6) tag with no arguments, which converts directly to HTML5 `<h6>`.",
    Header6Tag, ["h6"], "<h6>", "</h6>"
}
simple_tag!{
    "Centering tag with no arguments, which converts to a div with styling to horizontally center it.",
    CenterTag, ["center"], "<div style=\"display: flex; justify-content: center;\"><div>", "</div></div>"
}
simple_tag!{
    "Left-align tag with no arguments, which converts to a div with styling to left-align it.",
    LeftTag, ["left"], "<div style=\"display: flex; justify-content: left;\"><div>", "</div></div>"
}
simple_tag!{
    "Right-align tag with no arguments, which converts to a div with styling to right-align it.",
    RightTag, ["right"], "<div style=\"display: flex; justify-content: right;\"><div>", "</div></div>"
}
simple_tag!{
    "Preformatted styling tag with no arguments, which converts directly to HTML5 `<pre>`.",
    PreformattedTag, ["pre", "codeblock"], "<pre>", "</pre>"
}
simple_tag!{
    "Code styling tag with no arguments, which converts directly to HTML5 `<code>`.",
    CodeTag, ["code"], "<code>", "</code>"
}
simple_tag!{
    "Keypress styling tag with no arguments, which converts directly to HTML5 `<kbd>`.",
    KbdTag, ["kbd"], "<kbd>", "</kbd>"
}

macro_rules! tag_list {
    ($ct:ident; $($tag:ident),*) => {
        {
            let v: Vec<Box<dyn HtmlTagWriter<$ct>>> = vec![
                $(
                    Box::new($tag::default()),
                )*
            ];

            v
        }
    };
}

/// Returns all built-in tags from v1.0.0 (or earlier) of the library that are considered "basic" and safe for all usages by the authors.
/// # "Safe"
/// Safe, in the context of this list, is defined as 100% no doubts safe for using from random user input. This precludes things like image embeds, complex formatting, and links, which can be unsafe in some contexts and require special parsing.
/// # Included tags
/// - [BoldTag]
/// - [ItalicTag]
/// - [UnderlineTag]
/// - [LinebreakTag]
/// - [QuoteTag]
/// - [BlockQuoteTag]
/// - [SubscriptTag]
/// - [SuperscriptTag]
/// - [Header1Tag]
/// - [Header2Tag]
/// - [Header3Tag]
/// - [Header4Tag]
/// - [Header5Tag]
/// - [Header6Tag]
/// - [CenterTag]
/// - [CodeTag]
/// - [PreformattedTag]
/// - [KbdTag]
pub fn all_core_v1_tags<CustomTy>() -> Vec<Box<dyn HtmlTagWriter<CustomTy>>>
    where CustomTy: Clone + Default + 'static
{
    tag_list!{CustomTy;
        BoldTag,
        ItalicTag,
        UnderlineTag,
        LinebreakTag,
        QuoteTag,
        BlockQuoteTag,
        SubscriptTag,
        SuperscriptTag,
        Header1Tag,
        Header2Tag,
        Header3Tag,
        Header4Tag,
        Header5Tag,
        Header6Tag,
        CenterTag
    }
}
