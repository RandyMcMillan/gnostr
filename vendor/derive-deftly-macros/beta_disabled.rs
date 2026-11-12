//! Version of `mod beta` for when the cargo feature is disabled
//!
//! See `beta.rs` for dev documentation.

use super::prelude::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Enabled {}

pub fn with_maybe_enabled<R>(_: Option<Enabled>, f: impl FnOnce() -> R) -> R {
    f()
}

impl Enabled {
    pub fn new_for_dd_option(span: Span) -> syn::Result<Self> {
        Err(span.error(
 "derive-deftly's `beta_deftly` template option is only available when the `beta` cargo feature is also enabled"
        ))
    }

    #[allow(dead_code)] // sometimes we might not have any beta features
    pub fn new_for_syntax(span: Span) -> syn::Result<Self> {
        Err(span.error(
 "beta derive-deftly feature used, which requires both the `beta` cargo feature and the `beta_deftly` template option"
        ))
    }
}
