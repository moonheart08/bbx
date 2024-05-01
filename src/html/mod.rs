//! **Simple** HTML serialization from a [BBParser][crate::BBParser]'s output.
//!  While not comprehensive of more exotic usecases, likely sufficient for most and includes many pre-made tags.
use static_assertions::assert_obj_safe;

use crate::Token;

/// The primary trait for converting BBCode tags to HTML. 
pub trait HtmlTagGenerator<CustomTy = ()>
    where CustomTy: Clone
{
    /// Whether or not this generator is the correct generator for the given tag.
    /// # Remarks
    /// The implementation reserves the right to cache the result of this function on a per-instance basis, this function MUST 
    /// always return the same output for each possible input.
    fn match_tag(&self, tag: &str) -> bool;

    /// Produce an open tag for the given token, pushing it into the given buffer.
    /// # Remarks
    /// The implementation reserves the right to cache the result of this function on a per-instance basis, this function MUST 
    /// always push the same output for each possible input.
    /// The `out` buffer provided may already have contents, an implementation must not overwrite prior contents.
    fn open_tag<'a>(&self, token: &Token<'a, CustomTy>, out: &mut String);

    /// Produce a close tag for the given token, pushing it into the given buffer.
    /// # Remarks
    /// The implementation reserves the right to cache the result of this function on a per-instance basis, this function MUST 
    /// always push the same output for each possible input.
    /// The `out` buffer provided may already have contents, an implementation must not overwrite prior contents.
    fn close_tag<'a>(&self, open_token: &Token<'a, CustomTy>, close_token: &Token<'a, CustomTy>, out: &mut String);
}

assert_obj_safe!(HtmlTagGenerator);

/// 
pub trait HtmlTokenWriter<CustomTy = ()>
    where CustomTy: Clone
{

}