//! errors with compatibility advice
//!
//! Suitable for local glob import.
//!
//! ### Non-local errors from syn
//!
//! `syn` automatically produces "unexpected token" errors,
//! if not all of the input is consumed, somewhere.
//!
//! Empirically:
//! these errors are squirreled away somewhere, and surface
//! on return from one of the top-level syn `parse` functions
//! (the ones that provide a `ParseStream`).
//!
//! If the top-level function would return `Ok`,
//! the unexpected tokens error appears instead.
//! But if there's going to be an error anyway,
//! the unexpected tokens error is discarded.

use super::prelude::*;

/// A `Result` whose error might, or might not, need compat advice
///
///  * `Err(raw)`: unexpected error, probably mismatched
///    derive-deftly versions.
///    [`adviseable_parse2`] will return the error
///    but with compat advice for the user added.
///  * `Ok(ErrNeedsNoAdvice(cooked))`: "expected" error, fully reported for
///    the user's benefit.  Returned as-is by `parse_advised`.
///  * `Ok(Ok(T))`.  All went well.
///
/// This odd structure is to add a note to most of the errors that
/// come out of syn parsing.  The `braced!` etc. macros insist that the
/// calling scope throws precisely `syn::Error`; `Into<syn::Error>`
/// isn't enough.
///
/// This is the return type of `ParseAdviseable::parse_adviseable`.
//
// This is all rather unsatisfactory.  For example, it implies
// the AOk and ErrNNA type aliases and consequent ad-hoc glob imports
// of this module.  We'd prefer a custom error type, convertible from
// syn::Error, but syn won't allow that.
pub type AdviseableResult<T> = syn::Result<AdviseableInnerResult<T>>;

/// Typically found as `syn::Result<AdvisedResult<T>>`
#[derive(Debug)]
pub enum AdviseableInnerResult<T> {
    /// Success
    Ok(T),

    /// Failure, but doesn't need advice
    ///
    /// Typically found as
    /// `sync::Result::Ok(AdvisedInnerResult::NeedsNoAdvice(..))`.
    ErrNeedsNoAdvice(syn::Error),
}

/// Types that can be parsed, but might need compat advice
pub trait ParseAdviseable {
    fn parse_adviseable(input: ParseStream) -> AdviseableResult<Self>
    where
        Self: Sized;
}

pub use AdviseableInnerResult::ErrNeedsNoAdvice as ErrNNA;
pub use AdviseableInnerResult::Ok as AOk;

/// Parses with a callback, and produces advice if necessary
///
/// Version of `adviseable_parse2` that takes a callback function,
/// rather than a trait impl.
pub fn adviseable_parse2_call<T>(
    input: TokenStream,
    call: impl FnOnce(ParseStream) -> AdviseableResult<T>,
) -> syn::Result<T> {
    // If somehow our closure doesn't get called, we want to give
    // advice, so make that the default.
    let mut needs_advice = true;
    let ar = Parser::parse2(
        |input: ParseStream<'_>| {
            // When we're returning an error that needs no advice, we must
            // return *Err* from here, not Ok(NNA), because if we return Ok,
            // syn might surface an unexpected tokens error instead of the
            // non-advice-needing error we actually want.
            //
            // Encoding the advice-needed status in the error would
            // be difficult, given how opaque syn::Error is.  So
            // we smuggle a &mut bool into the closure.
            match call(input) {
                Err(needs) => Err(needs),
                Ok(ErrNNA(unadv)) => {
                    needs_advice = false;
                    Err(unadv)
                }
                Ok(AOk(y)) => Ok(y),
            }
        },
        input,
    );
    ar.map_err(|e| {
        if needs_advice {
            advise_incompatibility(e)
        } else {
            e
        }
    })
}

/// **Parses `T`, and produces advice if necessary** (principal entrypoint)
///
/// All top-level proc_macro entrypoints that want to give advice,
/// should ideally call this.
/// (See the note in `advise_incompatibility`.)
pub fn adviseable_parse2<T: ParseAdviseable>(
    input: TokenStream,
) -> syn::Result<T> {
    adviseable_parse2_call(input, T::parse_adviseable)
}

impl<T> AdviseableInnerResult<T> {
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> AdviseableInnerResult<U> {
        match self {
            AOk(y) => AOk(f(y)),
            ErrNNA(e) => ErrNNA(e),
        }
    }
}

/// Add a warning about derive-deftly version incompatibility
///
/// ### Lost advice hazard
///
/// Take care!
/// `syn` generates errors from unprocessed tokens in [`syn::parse2`] etc.
/// Calling this function *within* `syn::parse2`
/// (ie, somewhere you have a `ParseStream`,
/// will result in those errors not receiving advice.
///
/// Ideally, call this from functions that have a `TokenStream`.
/// If you do that, then functions *isnide* that can
/// use this method, avoiding the problem:
/// any errors stored up by `syn` will emerge at that call site,
/// and be properly dealt with.
pub fn advise_incompatibility(err_needing_advice: syn::Error) -> syn::Error {
    let mut advice = Span::call_site().error(
        "bad input to derive_deftly_engine inner template expansion proc macro; might be due to incompatible derive-deftly versions(s)"
    );
    advice.combine(err_needing_advice);
    advice
}

/// Within `parse_adviseable`, handle errors *without* giving advice
///
/// `parse_unadvised! { CONTENT_IDENT => || CLOSURE_BODY }`
/// expects `CONTENT_IDENT` to be the contents from
/// [`braced!`], [`bracketed!]` or [`parenthesized!`].
/// Calls the closure.
/// Errors within the closure won't get advice.
///
/// `parse_unadvised! { CONTENT_IDENT }`
/// shorthand for calling `.parse()` on the content.
///
/// # Sketch
///
/// ```rust,ignore
/// let something;
/// let _ = bracketed!(something in input);
/// parse_unadvised! {
///     something => || {
///         // do something with something, eg something.parse()
///         Ok(...)
///     }
/// }
/// ```
macro_rules! parse_unadvised { {
    $content:ident
} => {
    parse_unadvised! { $content => || $content.parse() }
}; {
    $content:ident => || $( $call:tt )*
} => {
    match syn::parse::Parser::parse2(
        // We convert the input to the `TokenStream2` and back,
        // so that we surface "unexpected token errors" here
        // rather than at the toplevel parsing call.
        |$content: ParseStream<'_>| -> syn::Result<_> {
            $($call)*
        },
        $content.parse::<TokenStream>()?
    ) {
        Ok(y) => y,
        Err::<_, syn::Error>(e) => return Ok(ErrNNA(e)),
    }
} }
