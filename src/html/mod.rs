//! **Simple** HTML serialization from a [BBParser]'s output.
//!  While not comprehensive of more exotic usecases, likely sufficient for most and includes many pre-made tags.
use std::collections::HashMap;

use static_assertions::assert_obj_safe;

use crate::{rules::ParserRuleObjBox, BBParser, Token, TokenKind};

/// The primary trait for converting BBCode tags to HTML.
pub trait HtmlTagWriter<CustomTy = ()>
where
    CustomTy: Clone + 'static,
{
    /// Whether or not this generator is the correct generator for the given tag.
    /// # Remarks
    /// The implementation reserves the right to cache the result of this function on a per-instance basis, this function MUST
    /// always return the same output for each possible input.
    fn match_tag(&self, tag: &str) -> bool;

    /// Produce an open tag for the given token, pushing it into the given buffer.
    /// # Remarks
    /// The `out` buffer provided may already have contents, an implementation must not overwrite prior contents.
    fn open_tag(
        &self,
        tk_writer: &dyn HtmlTokenWriter<CustomTy>,
        token: &Token<'_, CustomTy>,
        out: &mut String,
    );

    /// Produce a close tag for the given token, pushing it into the given buffer.
    /// # Remarks
    /// The `out` buffer provided may already have contents, an implementation must not overwrite prior contents.
    fn close_tag<'a>(
        &self,
        tk_writer: &dyn HtmlTokenWriter<CustomTy>,
        open_token: &Token<'a, CustomTy>,
        close_token: &Token<'a, CustomTy>,
        out: &mut String,
    );

    /// Produce a standalone tag for the given token, pushing it into the given buffer.
    /// # Remarks
    /// The `out` buffer provided may already have contents, an implementation must not overwrite prior contents.
    /// Standalone tags MUST NOT rely on other tags around them.
    fn standalone_tag(
        &self,
        tk_writer: &dyn HtmlTokenWriter<CustomTy>,
        token: &Token<'_, CustomTy>,
        out: &mut String,
    );

    /// Try to produce a new parser rule to introduce from the given token.
    fn try_special<'a>(
        &self,
        _token: &'_ Token<'a, CustomTy>,
    ) -> Option<ParserRuleObjBox<'a, CustomTy>> {
        None
    }
}

assert_obj_safe!(HtmlTagWriter);

/// Implements writing non-tag tokens.
pub trait HtmlTokenWriter<CustomTy = ()>
where
    CustomTy: Clone,
{
    /// Write HTML for the given token, pushing it into the given buffer.
    /// # Remarks
    /// The token is not guaranteed to only be text or custom tags, if a tag fails to match any tag writer it will end up here.
    /// The `out` buffer provided may already have contents, an implementation must not overwrite prior contents.
    fn write_token(&self, token: &Token<'_, CustomTy>, out: &mut String);
}

/// A dead simple HTML writer that simply html encodes the raw text of the tag it's given.
#[derive(Copy, Clone, Debug, Default)]
pub struct SimpleHtmlWriter;

impl HtmlTokenWriter<()> for SimpleHtmlWriter {
    fn write_token(&self, token: &Token<'_, ()>, out: &mut String) {
        out.push_str(&html_escape::encode_safe(token.span));
    }
}

/// Serializes a BBCode parse (from [BBParser]) to HTML using the registered tags and writer.
pub struct HtmlSerializer<Writer = SimpleHtmlWriter, CustomTy = ()>
where
    CustomTy: Clone,
    Writer: HtmlTokenWriter<CustomTy>,
{
    writer: Writer,
    tag_impls: Vec<Box<dyn HtmlTagWriter<CustomTy>>>,
    tag_cache: HashMap<String, usize>,
}

impl<Writer> HtmlSerializer<Writer>
where
    Writer: HtmlTokenWriter<()> + Default,
{
    /// Construct a new serializer with no tags.
    pub fn empty() -> Self {
        Self::with_tags(vec![])
    }

    /// Construct a new serializer with the given tags.
    pub fn with_tags(tags: Vec<Box<dyn HtmlTagWriter<()>>>) -> Self {
        Self::custom_with_tags(tags)
    }
}

impl<Writer, CustomTy> HtmlSerializer<Writer, CustomTy>
where
    CustomTy: Clone,
    Writer: HtmlTokenWriter<CustomTy> + Default,
{
    /// Construct a new serializer with the given tags.
    pub fn custom_with_tags(tags: Vec<Box<dyn HtmlTagWriter<CustomTy>>>) -> Self {
        Self::custom(tags, Writer::default())
    }
}

impl<Writer, CustomTy> HtmlSerializer<Writer, CustomTy>
where
    CustomTy: Clone,
    Writer: HtmlTokenWriter<CustomTy>,
{
    /// Construct a new serializer with the given tags and writer.
    pub fn custom(tag_impls: Vec<Box<dyn HtmlTagWriter<CustomTy>>>, writer: Writer) -> Self {
        Self {
            tag_impls,
            writer,
            tag_cache: Default::default(),
        }
    }
}

impl<Writer, CustomTy> HtmlSerializer<Writer, CustomTy>
where
    CustomTy: Clone + 'static,
    Writer: HtmlTokenWriter<CustomTy>,
{
    /// Serialize the given BBCode 'document' out to HTML, using the provided writer and tags.
    /// # Remarks
    /// This does not currently support out of order tags from [ParserFeature::POP_UNORDERED][crate::ParserFeature::POP_UNORDERED] and should not be used with it.
    pub fn serialize(&mut self, mut parser: BBParser<'_, CustomTy>) -> String {
        let mut out = String::with_capacity(parser.remaining().len());

        'outer: while let Some(tk) = parser.next() {
            match tk.kind {
                TokenKind::OpenBBTag(_)
                | TokenKind::CloseBBTag(_, Some(_))
                | TokenKind::StandaloneBBTag(_) => {
                    let Some(writer) = self.get_writer_for_tag(tk.tag_name().unwrap()) else {
                        self.writer.write_token(&tk, &mut out);
                        continue 'outer;
                    };

                    match tk.kind {
                        TokenKind::OpenBBTag(_) => writer.open_tag(&self.writer, &tk, &mut out),
                        TokenKind::CloseBBTag(_, Some(other)) => writer.close_tag(
                            &self.writer,
                            &parser.closed_tags()[other],
                            &tk,
                            &mut out,
                        ),
                        TokenKind::StandaloneBBTag(_) => {
                            writer.standalone_tag(&self.writer, &tk, &mut out)
                        }
                        _ => unreachable!(),
                    }

                    if let Some(r) = writer.try_special(&tk) {
                        parser.push_rule_obj(r);
                    }
                }
                _ => self.writer.write_token(&tk, &mut out),
            }
        }

        'outer: for tk in parser.open_tags() {
            // Handle any dangling tags.
            let Some(writer) = self.get_writer_for_tag(tk.tag_name().unwrap()) else {
                self.writer.write_token(tk, &mut out);
                continue 'outer;
            };

            let Token {
                kind: TokenKind::OpenBBTag(tag_data, ..),
                ..
            } = tk
            else {
                unreachable!()
            };

            let fake_close = Token {
                span: tk.span,
                start: tk.start,
                kind: TokenKind::CloseBBTag(tag_data.clone(), None),
            };

            writer.close_tag(&self.writer, tk, &fake_close, &mut out);
        }

        out
    }
}

impl<Writer, CustomTy> HtmlSerializer<Writer, CustomTy>
where
    CustomTy: Clone + 'static,
    Writer: HtmlTokenWriter<CustomTy>,
{
    /// Register the provided tags to the serializer.
    pub fn register_tags(&mut self, tags: &mut Vec<Box<dyn HtmlTagWriter<CustomTy>>>) {
        self.tag_impls.append(tags);
    }

    /// Register the provided tag to the serializer.
    pub fn register_tag(&mut self, tag: Box<dyn HtmlTagWriter<CustomTy>>) {
        self.tag_impls.push(tag);
    }

    /// Attempt to locate the implementation for the given tag, if one exists.
    pub fn get_writer_for_tag(&self, tag_name: &str) -> Option<&dyn HtmlTagWriter<CustomTy>> {
        if let Some(imp) = self.tag_cache.get(tag_name) {
            return Some(self.tag_impls[*imp].as_ref());
        }

        let idx: Option<usize> = 'idx: {
            for (idx, imp) in self.tag_impls.iter().enumerate() {
                if imp.match_tag(tag_name) {
                    break 'idx Some(idx);
                }
            }
            None
        };

        idx.map(|idx| self.tag_impls[idx].as_ref())
    }
}

pub mod builtins;

#[cfg(test)]
mod tests;
