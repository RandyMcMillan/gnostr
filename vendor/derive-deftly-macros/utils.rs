//! Utilities for proc macro implementation

use super::prelude::*;
use proc_macro_crate::{crate_name, FoundCrate};

//---------- dbg dumps of macro inputs/outputs ----------

/// Calls eprintln! but only if enabled
///
/// Enabling means *both* `--cfg derive_deftly_dprint`
/// and `DERIVE_DEFTLY_DPRINT` set to `1`.
///
/// ```text
/// RUSTFLAGS="--cfg derive_deftly_dprint" DERIVE_DEFTLY_DPRINT=1 cargo build
/// ```
///
/// Output can be rather interleaved due to test parallelism etc.
#[cfg(not(derive_deftly_dprint))]
#[macro_use]
pub mod dprint {
    macro_rules! dprintln { { $( $x:tt )* } => {} }

    /// For debug printing a block of input or output
    ///
    ///  * `dprint_block!( VALUE, FORMAT [, ARGS...]);`
    ///  * `dprint_block!( [VALUE, ...], FORMAT [, ARGS...]);`
    ///
    /// Each `VALUE` is formatting with `Display`.
    /// `FORMAT, [, ARGS]` is used in the dividers surrounding the output.
    macro_rules! dprint_block { { $( $x:tt )* } => {} }
}

#[cfg(derive_deftly_dprint)]
#[macro_use]
pub mod dprint {
    pub fn wanted() -> bool {
        const VAR: &str = "DERIVE_DEFTLY_DPRINT";
        match std::env::var_os(VAR) {
            None => false,
            Some(y) if y == "0" => false,
            Some(y) if y == "1" => true,
            other => panic!("bad value for {}: {:?}", VAR, other),
        }
    }

    macro_rules! dprintln { { $( $x:tt )* } => { {
        if dprint::wanted() {
            eprintln!( $($x)* )
        }
    } } }

    macro_rules! dprint_block { {
        [ $($value:expr),* $(,)? ], $f:literal $( $x:tt )*
    } => { {
        dprintln!(concat!("---------- ", $f, " start ----------") $($x)*);
        dprint_block!(@ $( $value, )*);
        dprintln!(concat!("---------- ", $f, " end ----------") $($x)*);
    } }; {
        @ $value:expr,
    } => { {
        dprintln!("{}", $value);
    } }; {
        @ $value:expr, $($more:tt)+
    } => { {
        dprint_block!(@ $value,);
        dprintln!("----------");
        dprint_block!(@ $($more)+);
    } }; {
        $value:expr, $f:literal $( $x:tt )*
    } => { {
        dprint_block!([$value], $f $($x)*);
    } } }
}

//---------- misc ----------

/// Construct a braced group from a token expansion
///
/// Calls `f` to expand tokens, and returns a braced group containing
/// its output.
pub fn braced_group(
    brace_span: Span,
    f: impl FnOnce(&mut TokenAccumulator) -> syn::Result<()>,
) -> syn::Result<proc_macro2::Group> {
    delimit_token_group(Delimiter::Brace, brace_span, f)
}

pub fn delimit_token_group(
    delim: Delimiter,
    delim_span: Span,
    f: impl FnOnce(&mut TokenAccumulator) -> syn::Result<()>,
) -> syn::Result<proc_macro2::Group> {
    let mut out = TokenAccumulator::default();
    f(&mut out)?;
    let out = out.tokens()?;
    let mut out = proc_macro2::Group::new(delim, out);
    out.set_span(delim_span);
    Ok(out)
}

/// Type which parses as `T`, but then discards it
#[allow(dead_code)] // used in tests
pub struct Discard<T>(PhantomData<T>);

impl<T: Parse> Parse for Discard<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _: T = input.parse()?;
        Ok(Discard(PhantomData))
    }
}

/// Type which parses as a concatenated series of `T`
#[derive(Debug, Clone)]
#[allow(dead_code)] // used in tests and certain configurations
pub struct Concatenated<T>(pub Vec<T>);

impl<T: Parse> Parse for Concatenated<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut out = vec![];
        while !input.is_empty() {
            out.push(input.parse()?);
        }
        Ok(Concatenated(out))
    }
}

impl<T: ToTokens> ToTokens for Concatenated<T> {
    fn to_tokens(&self, out: &mut TokenStream) {
        for item in &self.0 {
            item.to_tokens(out);
        }
    }
}

/// Add any missing colons before `< >`'d generic arguments
///
/// ```rust,ignore
/// std::option::Option::<>
/// //                 ^^ these colons
/// ```
pub fn typepath_add_missing_argument_colons(
    type_path: &mut syn::TypePath,
    te_span: Span,
) {
    for seg in &mut type_path.path.segments {
        match &mut seg.arguments {
            syn::PathArguments::None => {}
            syn::PathArguments::Parenthesized(_) => {}
            syn::PathArguments::AngleBracketed(args) => {
                args.colon2_token.get_or_insert_with(|| {
                    // Use template's span in case this somehow
                    // produces a syntax error.  The actual
                    // names have the names' spans.
                    Token![::](te_span)
                });
            }
        }
    }
}

pub fn dummy_path() -> syn::Path {
    syn::Path {
        leading_colon: None,
        segments: Punctuated::default(),
    }
}

//---------- MakeError ----------

/// Provides `.error()` on `impl Spanned` and `[`[`ErrorLoc`]`]`
pub trait MakeError {
    /// Convenience method to make an error
    fn error<M: Display>(&self, m: M) -> syn::Error;
}

impl<T: Spanned> MakeError for T {
    fn error<M: Display>(&self, m: M) -> syn::Error {
        syn::Error::new(self.span(), m)
    }
}

/// Error location: span and what role that span plays
///
/// When `MakeError::error` is invoked on a slice of these,
/// the strings are included to indicate to the user
/// what role each `Span` location played.
/// For example, `(tspan, "template")`.
pub type ErrorLoc<'s> = (Span, &'s str);

/// Generates multiple copies of the error, for multiple places
///
/// # Panics
///
/// Panics if passed an empty slice.
impl MakeError for [ErrorLoc<'_>] {
    fn error<M: Display>(&self, m: M) -> syn::Error {
        let mut locs = self.into_iter().cloned();
        let mk = |(span, frag): (Span, _)| {
            span.error(format_args!("{} ({})", &m, frag))
        };
        let first = locs.next().expect("at least one span needed!");
        let mut build = mk(first);
        for rest in locs {
            build.combine(mk(rest))
        }
        build
    }
}

//---------- Span::join with fallback ----------

/// Returns a span covering the inputs
///
/// Sometimes this isn't possible (see [`Span::join`],
/// in which case prefers the span of the final component.
///
/// Returns `Some` iff the iterator was nonempty.
pub fn spans_join(spans: impl IntoIterator<Item = Span>) -> Option<Span> {
    spans.into_iter().reduce(|a, b| a.join(b).unwrap_or(b))
}

//---------- ToTokensPunctComposable ----------

/// Convert to a token stream in a way that composes nicely
pub trait ToTokensPunctComposable {
    fn to_tokens_punct_composable(&self, out: &mut TokenStream);
}
/// Ensure that there is a trailing punctuation if needed
impl<T, P> ToTokensPunctComposable for Punctuated<T, P>
where
    T: ToTokens,
    P: ToTokens + Default,
{
    fn to_tokens_punct_composable(&self, out: &mut TokenStream) {
        self.to_tokens(out);
        if !self.empty_or_trailing() {
            P::default().to_tokens(out)
        }
    }
}
/// Ensure that something is output, for punctuation `P`
///
/// Implemented for `Option<&&P>` because that's what you get from
/// `Punctuated::pairs().next().punct()`.
impl<P> ToTokensPunctComposable for Option<&&P>
where
    P: ToTokens,
    P: Default,
{
    fn to_tokens_punct_composable(&self, out: &mut TokenStream) {
        if let Some(self_) = self {
            self_.to_tokens(out)
        } else {
            P::default().to_tokens(out)
        }
    }
}

//---------- ErrorAccumulator ----------

/// Contains zero or more `syn::Error`
///
/// # Panics
///
/// Panics if dropped.
///
/// You must call one of the consuming methods, eg `finish`
#[derive(Debug, Default)]
pub struct ErrorAccumulator {
    bad: Option<syn::Error>,
    defused: bool,
}

impl ErrorAccumulator {
    /// Run `f`, accumulate any error, and return an `Ok`
    pub fn handle_in<T, F>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce() -> syn::Result<T>,
    {
        self.handle(f())
    }

    /// Handle a `Result`: accumulate any error, and returni an `Ok`
    pub fn handle<T>(&mut self, result: syn::Result<T>) -> Option<T> {
        match result {
            Ok(y) => Some(y),
            Err(e) => {
                self.push(e);
                None
            }
        }
    }

    /// Accumulate an error
    pub fn push(&mut self, err: syn::Error) {
        if let Some(bad) = &mut self.bad {
            bad.combine(err)
        } else {
            self.bad = Some(err);
        }
    }

    /// If there were any errors, return a single error that combines them
    #[allow(dead_code)]
    pub fn finish(self) -> syn::Result<()> {
        self.finish_with(())
    }

    /// If there were any errors, return `Err`, otherwise `Ok(success)`
    pub fn finish_with<T>(self, success: T) -> syn::Result<T> {
        match self.into_inner() {
            None => Ok(success),
            Some(bad) => Err(bad),
        }
    }

    /// If there any errors, return a single error that combines them
    pub fn into_inner(mut self) -> Option<syn::Error> {
        self.defused = true;
        self.bad.take()
    }
}

impl Drop for ErrorAccumulator {
    fn drop(&mut self) {
        assert!(panicking() || self.defused);
    }
}

//---------- Template and driver export ----------

/// Token `export` (or `pub`), indicating that a macro should be exported
///
/// Usually found in `Option`.
#[derive(Debug, Clone)]
pub struct MacroExport(Span);

impl Spanned for MacroExport {
    fn span(&self) -> Span {
        self.0
    }
}

impl MacroExport {
    pub fn parse_option(input: ParseStream) -> syn::Result<Option<Self>> {
        let span = if let Some(vis) = input.parse::<Option<Token![pub]>>()? {
            return Err(vis.error(
 "You must now write `define_derive_deftly! { export Template: ... }`, not `puib Template:`, since derive-deftly version 0.14.0"
            ));
        } else if let Some(export) = (|| {
            use syn::parse::discouraged::Speculative;
            input.peek(syn::Ident).then(|| ())?;
            let forked = input.fork();
            let ident: syn::Ident = forked.parse().expect("it *was*");
            (ident == "export").then(|| ())?;
            input.advance_to(&forked);
            Some(ident)
        })() {
            Some(export.span())
        } else {
            None
        };
        Ok(span.map(MacroExport))
    }
}

//---------- Grouping ----------

/// Whether an expansion should be surrounded by a `None`-delimited `Group`
///
/// `Ord` is valid for composition with `cmp::max`
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Grouping {
    Ungrouped,
    Invisible,
    Parens,
}

impl Grouping {
    pub fn surround(&self, ts: impl ToTokens) -> TokenStream {
        let ts = ts.to_token_stream();
        match self {
            Grouping::Ungrouped => ts,
            Grouping::Invisible => {
                proc_macro2::Group::new(Delimiter::None, ts).to_token_stream()
            }
            Grouping::Parens => {
                proc_macro2::Group::new(Delimiter::Parenthesis, ts)
                    .to_token_stream()
            }
        }
    }
}

//---------- IdentAny ----------

/// Like `syn::Ident` but parses using `parse_any`, accepting keywords
///
/// Used for derive-deftly's own keywords, which can be Rust keywords,
/// or identifiers.
///
/// Needs care when used with user data, since it might be a keyword,
/// in which case it's not really an *identifier*.
pub struct IdentAny(pub syn::Ident);
impl Parse for IdentAny {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(IdentAny(Ident::parse_any(input)?))
    }
}
impl Deref for IdentAny {
    type Target = syn::Ident;
    fn deref(&self) -> &syn::Ident {
        &self.0
    }
}
impl ToTokens for IdentAny {
    fn to_tokens(&self, out: &mut TokenStream) {
        self.0.to_tokens(out)
    }
}
impl<T: AsRef<str> + ?Sized> PartialEq<T> for IdentAny {
    fn eq(&self, rhs: &T) -> bool {
        self.0.eq(rhs)
    }
}

//---------- OutputTrimmer ----------

/// For making an output TokenStream, but eliding an unnecessary tail
///
/// This construction will write, to an output [`TokenStream`],
///
///  * `preamble`
///  * zero or more optional `impl ToTokens`, called "reified"
///  * interleaved with zero or more optional separators `sep`
///
/// But it will avoid writing trailing unnecessary content:
/// that is, trailing calls to `push_sep` are ignored,
/// and if `push_reified` is never called,
/// the preamble is also omitted.
pub struct TokenOutputTrimmer<'t, 'o> {
    preamble: Option<&'t dyn ToTokens>,
    sep: &'t dyn ToTokens,
    sep_count: usize,
    out: &'o mut TokenStream,
}

impl<'t, 'o> TokenOutputTrimmer<'t, 'o> {
    pub fn new(
        out: &'o mut TokenStream,
        preamble: &'t dyn ToTokens,
        sep: &'t dyn ToTokens,
    ) -> Self {
        TokenOutputTrimmer {
            preamble: Some(preamble),
            sep,
            sep_count: 0,
            out,
        }
    }
    pub fn push_sep(&mut self) {
        self.sep_count += 1;
    }
    fn reify(&mut self) {
        if let Some(preamble) = self.preamble.take() {
            preamble.to_tokens(&mut self.out);
        }
        for _ in 0..mem::take(&mut self.sep_count) {
            self.sep.to_tokens(&mut self.out);
        }
    }
    pub fn push_reified(&mut self, t: impl ToTokens) {
        self.reify();
        t.to_tokens(&mut self.out);
    }
    /// Did we output the preamble at all?
    pub fn did_preamble(self) -> Option<()> {
        if self.preamble.is_some() {
            None
        } else {
            Some(())
        }
    }
}

//---------- TemplateName ----------

#[derive(Debug, Clone)]
pub struct TemplateName(syn::Ident);

impl TemplateName {
    pub fn macro_name(&self) -> syn::Ident {
        format_ident!("derive_deftly_template_{}", &self.0)
    }
}

impl Parse for TemplateName {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        ident.try_into()
    }
}
impl TryFrom<syn::Ident> for TemplateName {
    type Error = syn::Error;
    fn try_from(ident: syn::Ident) -> syn::Result<TemplateName> {
        let s = ident.to_string();
        match s.chars().find(|&c| c != '_') {
            None => {
                Err("template name cannot consist entirely of underscores")
            }
            Some(c) => {
                if c.is_lowercase() {
                    Err(
 "template name may not start with a lowercase letter (after any underscores)"
                )
                } else {
                    Ok(())
                }
            }
        }
        .map_err(|emsg| ident.error(emsg))?;

        Ok(TemplateName(ident))
    }
}
impl ToTokens for TemplateName {
    fn to_tokens(&self, out: &mut TokenStream) {
        self.0.to_tokens(out)
    }
}
impl Display for TemplateName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}
impl quote::IdentFragment for TemplateName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        quote::IdentFragment::fmt(&self.0, f)
    }
    fn span(&self) -> Option<Span> {
        quote::IdentFragment::span(&self.0)
    }
}

//---------- engine_macro_name ----------

/// Return a full path to the location of `derive_deftly_engine`.
///
/// (This may not work properly if the user
/// imports the crate under a different name.
/// This is a problem with the way cargo and rustc
/// handle imports and proc-macro crates,
/// which I think we can't properly solve here.)
pub fn engine_macro_name() -> Result<TokenStream, syn::Error> {
    let name = crate_name("derive-deftly-macros")
        .or_else(|_| crate_name("derive-deftly"));

    // See `tests/pub-export/pub-b/pub-b.rs`.  (The bizarre version
    // has a different crate name, which we must handle heree.)
    #[cfg(feature = "bizarre")]
    let name = name.or_else(|_| crate_name("bizarre-derive-deftly"));

    match name {
        Ok(FoundCrate::Itself) => Ok(quote!( crate::derive_deftly_engine )),
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            Ok(quote!( ::#ident::derive_deftly_engine ))
        }
        Err(e) => Err(Span::call_site().error(
            format_args!("Expected derive-deftly or derive-deftly-macro to be present in Cargo.toml: {}", e)
        )),
    }
}

//---------- general keyword enum parsing ----------

/// General-purpose keyword parser
///
/// ```ignore
/// keyword_general!{
///     KW_VAR FROM_ENUM ENUM;
///     KEYWORD [ {BINDINGS} ] [ CONSTRUCTOR-ARGS ] }
/// ```
/// Expands to:
/// ```ignore
///     if KW_VAR = ... {
///         BINDINGS
///         return FROM_ENUM(ENUM::CONSTRUCTOR CONSTRUCTOR-ARGS)
///     }
/// ```
///
/// `KEYWORD` can be `"KEYWORD_STRING": CONSTRUCTOR`
///
/// `CONSTRUCTOR-ARGS`, if present, should be in the `( )` or `{ }`
/// as required by the variant's CONSTRUCTOR.
macro_rules! keyword_general {
    { $kw_var:ident $from_enum:ident $Enum:ident;
      $kw:ident $( $rest:tt )* } => {
        keyword_general!{ $kw_var $from_enum $Enum;
                          @ 1 stringify!($kw), $kw, $($rest)* }
    };
    { $kw_var:ident $from_enum:ident $Enum:ident;
      $kw:literal: $constr:ident $( $rest:tt )* } => {
        keyword_general!{ $kw_var $from_enum $Enum;
                          @ 1 $kw, $constr, $($rest)* }
    };
    { $kw_var:ident $from_enum:ident $Enum:ident;
      @ 1 $kw:expr, $constr:ident, $( $ca:tt )? } => {
        keyword_general!{ $kw_var $from_enum $Enum;
                          @ 2 $kw, $constr, { } $( $ca )? }
    };
    { $kw_var:ident $from_enum:ident $Enum:ident;
      @ 1 $kw:expr, $constr:ident, { $( $bindings:tt )* } $ca:tt } => {
        keyword_general!{ $kw_var $from_enum $Enum;
                          @ 2 $kw, $constr, { $( $bindings )* } $ca }
    };
    { $kw_var:ident $from_enum:ident $Enum:ident;
      @ 2 $kw:expr, $constr:ident,
      { $( $bindings:tt )* } $( $constr_args:tt )?
    } => {
        let _: &IdentAny = &$kw_var;
        if $kw_var == $kw {
            $( $bindings )*
            return $from_enum($Enum::$constr $( $constr_args )*);
        }
    };
    { $($x:tt)* } => { compile_error!(stringify!($($x)*)) };
}
