//! Built-in implementations of common (i.e. used by many implementations, subjectively) BBCode tags.

use super::HtmlTagWriter;

mod link;
mod simple;
pub use link::*;
pub use simple::*;

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
/// - [StrikethroughTag]
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
where
    CustomTy: Clone + Default + 'static,
{
    tag_list! {CustomTy;
        BoldTag,
        ItalicTag,
        UnderlineTag,
        StrikethroughTag,
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
