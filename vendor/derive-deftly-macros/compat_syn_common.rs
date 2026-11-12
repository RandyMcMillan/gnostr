//! Definitions for compatibility with syn, common to 1 and 2

use super::prelude::*;

//---------- Attribute handling ----------

/// Helper trait to deal with syn's idea of attribute contents
///
/// We expect all our attributes to be parenthesised.
pub trait AttributeExt12 {
    /// Parse the content within an `#[attribute(content)]`
    ///
    /// Parses the `content` as `T`.
    fn parse_in_parens<T: Parse>(&self) -> syn::Result<T> {
        self.call_in_parens(Parse::parse)
    }

    /// Like `parse_in_parens`, but takes a parser callback.
    fn call_in_parens<T, F>(&self, f: F) -> syn::Result<T>
    where
        F: FnOnce(ParseStream<'_>) -> syn::Result<T>;
}
