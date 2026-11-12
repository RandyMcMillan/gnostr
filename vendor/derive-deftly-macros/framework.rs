//! Core types and traits for parsing and expansion
//!
//! Also re-exports the names that the implementation wants.
//!
//! Should be included with `use super::framework::*`, not `crate::`,
//! so that it works with `tests/directly.rs` too.

pub use super::prelude::*;

pub use super::boolean::*;
pub use super::repeat::*;
pub use super::syntax::*;

pub(super) use super::paste;
pub(super) use super::paste::{IdentFrag, IdentFragInfallible};

/// Context during expansion
///
/// References the driver, and digested information about it.
/// Also represents where in the driver we are,
/// including repetition context.
#[derive(Debug, Clone)]
pub struct Context<'c> {
    pub top: &'c syn::DeriveInput,
    pub template_crate: &'c syn::Path,
    pub template_name: Option<&'c syn::Path>,
    pub pmetas: &'c meta::PreprocessedMetas,
    pub variant: Option<&'c WithinVariant<'c>>,
    pub field: Option<&'c WithinField<'c>>,
    pub within_loop: WithinLoop,
    pub pvariants: &'c [PreprocessedVariant<'c>],
    pub definitions: Definitions<'c>,
    pub nesting_depth: u16,
    pub nesting_parent: Option<(&'c Context<'c>, &'c DefinitionName)>,
}

#[derive(Debug)]
pub struct PreprocessedVariant<'c> {
    pub fields: &'c syn::Fields,
    pub pmetas: &'c meta::PreprocessedMetas,
    pub pfields: Vec<PreprocessedField>,
}

#[derive(Debug)]
pub struct PreprocessedField {
    pub pmetas: meta::PreprocessedMetas,
}

#[derive(Debug, Clone)]
pub struct WithinVariant<'c> {
    pub variant: Option<&'c syn::Variant>,
    pub fields: &'c syn::Fields,
    pub pmetas: &'c meta::PreprocessedMetas,
    pub pfields: &'c [PreprocessedField],
}

#[derive(Debug, Clone)]
pub struct WithinField<'c> {
    pub field: &'c syn::Field,
    pub pfield: &'c PreprocessedField,
    pub index: u32,
}

/// Whether we're in a loop, and if so, its details
///
/// Set only for expansions of a `RepeatedTemplate`,
/// not any kind of implicit looping eg `dbg_all_keywords`, `vpat`, etc.
///
/// At some future point this may have enough information
/// to provide `$loop_index`, etc.
/// Right now it's only used for `dbg_all_keywords`.
#[derive(Debug, Clone, Copy)]
pub enum WithinLoop {
    None,
    /// Evaluating `${when }` clauses
    When,
    /// Evaluating the body
    Body,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Definitions<'c> {
    pub here: &'c [&'c Definition<DefinitionBody>],
    pub conds: &'c [&'c Definition<DefCondBody>],
    pub earlier: Option<&'c Definitions<'c>>,
}

/// Special processing instructions returned by
/// [`special_before_element_hook`](SubstParseContext::special_before_element_hook)
pub enum SpecialInstructions {
    /// This template is finished
    ///
    /// Stop parsing this `Template` though perhaps
    /// the surrounding `Group` is not finished.
    ///
    /// The parser for whatever called `Template::parse`
    /// will continue.
    EndOfTemplate,
}

/// Surrounding lexical context during parsing
///
/// This is the kind of lexical context a piece of a template appears in.
/// It is implemented for
///  * Types that represent an expansion output `ExpansionOutput`;
///    in this case, the lexical context is one where
///    the expansion is accumulated in this type.
///  * Places where template substitution syntax `${keyword }`
///    appears but where no output will be generated (eg, within
///    the condition of `${if }`.
///
/// The associated types are either `Void` or `()`.
/// They appears within the variants of `SubstDetails`,
/// causing inapplicable variants to be eliminated.
///
/// Because a variant is only inhabited if all of its fields are,
/// the conditions are effectively ANDed.
/// So the "default" value (for context that don't have an opnion)
/// is inhabitedness `()`.
///
/// Each type has an associated constructur,
/// used during parsing.
/// So this generates a parse error at parse time,
/// if a construct appears in the wrong place.
pub trait SubstParseContext: Sized {
    /// Uninhabited iff this lexical context is within `${paste }`
    type NotInPaste: Debug + Copy + Sized;
    /// Uninhabited iff this lexical context is within a condition.
    type NotInBool: Debug + Copy + Sized;
    /// Uninhabited unless this lexical context is within a condition.
    type BoolOnly: Debug + Copy + Sized;

    /// Whether this is a boolean context
    //
    // Useful for ad-hoc handling of the way that boolean
    // context has a different notion of syntax.
    const IS_BOOL: bool = false;

    /// Content of the `dbg` keyword
    ///
    /// This has to be in this trait because
    /// `${dbg }` contains a `Template` but `dbg(...)` contains a `Subst`.
    ///
    /// We make bespoke output for each context; for boolean this is sui
    /// generis, and for expansions it's in [`ExpansionOutput::dbg_expand`].
    type DbgContent: Parse + Debug + AnalyseRepeat + meta::FindRecogMetas;

    fn not_in_paste(span: &impl Spanned) -> syn::Result<Self::NotInPaste>;
    fn not_in_bool(span: &impl Spanned) -> syn::Result<Self::NotInBool>;

    fn bool_only(span: &impl Spanned) -> syn::Result<Self::BoolOnly> {
        Err(span.error(
            "derive-deftly keyword is a condition - not valid as an expansion",
        ))
    }

    /// When we find a `fmeta` etc. in this context, does it allow a value?
    ///
    /// Used by the template-scanning code, to report whether an `Xmeta`
    /// in the template justifies a value-bearing `Xmeta` attribute
    /// on/in the driver, or just a boolean.
    fn meta_recog_usage() -> meta::Usage;

    /// For communicating through `parse_special`
    type SpecialParseContext: Default;

    /// Handle any special syntax for a special kind of template context.
    ///
    /// This method is called only when parsing multi-element [`Template`]s,
    /// It's a hook, called before parsing each `TemplateElement`.
    ///
    /// It should consume any special syntax as appropriate,
    ///
    /// The default implementation is a no-op.
    /// The only non-default implementation is in `paste.rs`, for `$<...>` -
    /// see [`paste::AngleBrackets`].
    fn special_before_element_hook(
        _special: &mut Self::SpecialParseContext,
        _input: ParseStream,
    ) -> syn::Result<Option<SpecialInstructions>> {
        Ok(None)
    }

    /// Parse using `f`, within parens in boolean context, not otherwise
    ///
    /// Useful for parsing the arguments to an argument-taking keyword
    /// which takes an "equivalent" syntax in both contexts.
    fn parse_maybe_within_parens<T>(
        input: ParseStream,
        f: impl FnOnce(ParseStream) -> syn::Result<T>,
    ) -> syn::Result<T> {
        if Self::IS_BOOL {
            let inner;
            let _ = parenthesized!(inner in input);
            f(&inner)
        } else {
            f(input)
        }
    }

    /// Parse maybe a comma (comma in boolean contegxt, not otherwise)
    ///
    /// Useful for parsing the arguments to an argument-taking keyword
    /// which takes an "equivalent" syntax in both contexts.
    fn parse_maybe_comma(input: ParseStream) -> syn::Result<()> {
        if Self::IS_BOOL {
            let _: Token![,] = input.parse()?;
        }
        Ok(())
    }

    /// Return an error suitable for reporting missing arguments
    ///
    /// Helper for handling missing arguments to an argument-taking keyword
    /// which takes an "equivalent" syntax in both contexts.
    fn missing_keyword_arguments(kw_span: Span) -> syn::Result<Void> {
        Err(kw_span.error(format_args!(
 "missing parameters to expansion keyword (NB: argument must be within {{ }})",
        )))
    }
}

/// Expansion output accumulator, for a template lexical context
///
/// Each template lexical context has a distinct type which
///  * Represents the lexical context
///  * If that lexical context generates expansions,
///    accumulates the expansion.  That's what this trait is.
///
/// The methods are for accumulating various kinds of things
/// that can be found in templates, or result from template expansion.
///
/// The accumulating type (`Self` might be accumulating
/// tokens ([`TokenStream`]) or strings ([`paste::Items`]).
pub trait ExpansionOutput: SubstParseContext {
    /// An identifier (or fragment of one)
    ///
    /// Uses the `IdentFragment` for identifier pasting,
    /// and the `ToTokens` for general expansion.
    fn append_identfrag_toks<I: IdentFrag>(
        &mut self,
        ident: &I,
    ) -> Result<(), I::BadIdent>;

    /// Append a Rust path (scoped identifier, perhaps with generics)
    ///
    /// To facilitate `${pawte }`, the path is provided as:
    ///  * some prefix tokens (e.g., a scoping path),
    ///  * the actual identifer,
    ///  * some suffix tokens (e.g. generics).
    ///
    /// `tspan` is the span of the part of the template
    /// which expanded into this path.
    ///
    /// This is a "more complex" expansion,
    /// in the terminology of the template reference:
    /// If a paste contains more than one, it is an error.
    fn append_idpath<A, B, I>(
        &mut self,
        template_entry_span: Span,
        pre: A,
        ident: &I,
        post: B,
        grouping: Grouping,
    ) -> Result<(), I::BadIdent>
    where
        A: FnOnce(&mut TokenAccumulator),
        B: FnOnce(&mut TokenAccumulator),
        I: IdentFrag;

    /// Append a [`syn::LitStr`](struct@syn::LitStr)
    ///
    /// This is its own method because `syn::LitStr` is not `Display`,
    /// and we don't want to unconditionally turn it into a string
    /// before retokenising it.
    fn append_syn_litstr(&mut self, v: &syn::LitStr);

    /// Append a [`syn::Type`]
    ///
    /// This is a "more complex" expansion,
    /// in the terminology of the template reference:
    /// If a paste contains more than one, it is an error.
    fn append_syn_type(
        &mut self,
        te_span: Span,
        mut v: syn::Type,
        mut grouping: Grouping,
    ) {
        loop {
            let (inner, add_grouping) = match v {
                syn::Type::Paren(inner) => (inner.elem, Grouping::Parens),
                syn::Type::Group(inner) => (inner.elem, Grouping::Invisible),
                _ => break,
            };
            v = *inner;
            grouping = cmp::max(grouping, add_grouping);
        }
        if let syn::Type::Path(tp) = &mut v {
            typepath_add_missing_argument_colons(tp, te_span);
        }
        self.append_syn_type_inner(te_span, v, grouping)
    }

    /// Append a [`syn::Type`], which has been grouping-normalised
    fn append_syn_type_inner(
        &mut self,
        te_span: Span,
        v: syn::Type,
        grouping: Grouping,
    );

    /// Append using a function which generates tokens
    ///
    /// If you have an `impl `[`ToTokens`],
    /// use [`append_tokens`](ExpansionOutput::append_tokens) instead.
    ///
    /// Not supported within `${paste }`.
    /// The `NotInPaste` parameter makes this method unreachable
    /// when expanding within `${paste }`;
    /// or to put it another way,
    /// it ensures that such an attempt would have been rejected
    /// during template parsing.
    fn append_tokens_with(
        &mut self,
        np: &Self::NotInPaste,
        f: impl FnOnce(&mut TokenAccumulator) -> syn::Result<()>,
    ) -> syn::Result<()>;

    /// "Append" a substitution which can only be used within a boolean
    ///
    /// Such a thing cannot be expanded, so it cannot be appended,
    /// so this function must be unreachable.
    /// `expand_bool_only` is called (in expansion contexts)
    /// to handle uninhabited `SubstDetails` variants etc.
    ///
    /// Implementing it involves demonstrating that
    /// either `self`, or `Self::BoolOnly`, is uninhabited,
    /// with a call to [`void::unreachable`].
    fn append_bool_only(&mut self, bool_only: &Self::BoolOnly) -> !;

    /// Note that an error occurred
    ///
    /// This must arrange to
    /// (eventually) convert it using `into_compile_error`
    /// and emit it somewhere appropriate.
    fn record_error(&mut self, err: syn::Error);

    /// Convenience method for noting an error with span and message
    fn write_error<S: Spanned, M: Display>(&mut self, span: &S, message: M) {
        self.record_error(span.error(message));
    }

    /// Convenience method for writing a `ToTokens`
    ///
    /// Dispatches to
    /// [`append_tokens_with`](ExpansionOutput::append_tokens_with)
    /// Not supported within `${paste }`.
    //
    // I experimented with unifying this with `append_tokens_with`
    // using a `ToTokensFallible` trait, but it broke type inference
    // rather badly and had other warts.
    fn append_tokens(
        &mut self,
        np: &Self::NotInPaste,
        tokens: impl ToTokens,
    ) -> syn::Result<()> {
        self.append_tokens_with(np, |out| {
            out.append(tokens);
            Ok(())
        })
    }

    /// Make a new empty expansion output, introduced at `kw_span`
    ///
    /// Normally, call sites use an inherent constructor method.
    /// This one is used for special cases, eg `${ignore ...}`
    fn new_with_span(kw_span: Span) -> Self;

    fn default_subst_meta_as(kw: Span) -> syn::Result<meta::SubstAs<Self>>;

    /// Implement the core of the `ignore` keyword
    ///
    /// If there was an error, returns it.
    /// Otherwise, discards everything.
    fn ignore_impl(self) -> syn::Result<()>;

    /// Implement the `dbg` keyword
    ///
    /// Specifically:
    ///   * Write the expansion of `child` to `msg` in human-readable form
    ///   * Without a trailing newline
    ///   * Subsume it into `self`
    ///
    /// Failures are to be reported, and subsumed into `self`.
    fn dbg_expand(
        &mut self,
        kw_span: Span,
        ctx: &Context,
        msg: &mut String,
        content: &Self::DbgContent,
    );
}

/// Convenience trait providing `item.expand()`
///
/// Implementations of this are often specific to the [`ExpansionOutput`].
///
/// Having this as a separate trait,
/// rather than hanging it off `ExpansionOutput`,
/// makes the expansion method more convenient to call.
///
/// It also avoids having to make all of these expansion methods
/// members of the `ExpansionOutput` trait.
pub trait Expand<O> {
    fn expand(&self, ctx: &Context, out: &mut O) -> syn::Result<()>;
}
/// Convenience trait providing `fn expand(self)`, infallible version
///
/// Some of our `expand` functions always record errors
/// within the output accumulator
/// and therefore do not need to return them.
pub trait ExpandInfallible<O> {
    fn expand(&self, ctx: &Context, out: &mut O);
}

/// Accumulates tokens, or errors
///
/// We collect all the errors, and if we get an error, don't write
/// anything out.
/// This is because `compile_error!` (from `into_compile_error`)
/// only works in certain places in Rust syntax (!)
#[derive(Debug)]
pub struct TokenAccumulator(Result<TokenStream, syn::Error>);

impl<'c> Context<'c> {
    pub fn is_enum(&self) -> bool {
        matches!(self.top.data, syn::Data::Enum(_))
    }

    /// Description of the whole expansion, suitable for `dbg` option, etc.
    pub fn expansion_description(&self) -> impl Display {
        let ident = &self.top.ident;
        if let Some(templ) = &self.template_name {
            format!(
                "derive-deftly expansion of {} for {}",
                templ.to_token_stream(),
                ident,
            )
        } else {
            format!("derive-deftly expansion, for {}", ident,)
        }
    }
}

impl Default for TokenAccumulator {
    fn default() -> Self {
        TokenAccumulator(Ok(TokenStream::new()))
    }
}

impl TokenAccumulator {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_tokens<R>(
        &mut self,
        f: impl FnOnce(&mut TokenStream) -> R,
    ) -> Option<R> {
        self.0.as_mut().ok().map(f)
    }
    pub fn append(&mut self, t: impl ToTokens) {
        self.with_tokens(|out| t.to_tokens(out));
    }
    pub fn tokens(self) -> syn::Result<TokenStream> {
        self.0
    }
    /// Appends `val`, via [`ToTokensPunctComposable`] or [`ToTokens`]
    pub fn append_maybe_punct_composable(
        &mut self,
        val: &(impl ToTokens + ToTokensPunctComposable),
        composable: bool,
    ) {
        self.with_tokens(|out| {
            if composable {
                val.to_tokens_punct_composable(out);
            } else {
                val.to_tokens(out);
            }
        });
    }
}

impl SubstParseContext for TokenAccumulator {
    type NotInPaste = ();
    type NotInBool = ();
    type DbgContent = Template<TokenAccumulator>;
    fn not_in_bool(_: &impl Spanned) -> syn::Result<()> {
        Ok(())
    }
    fn not_in_paste(_: &impl Spanned) -> syn::Result<()> {
        Ok(())
    }
    fn meta_recog_usage() -> meta::Usage {
        meta::Usage::Value
    }

    type BoolOnly = Void;

    type SpecialParseContext = ();
}

impl ExpansionOutput for TokenAccumulator {
    fn append_identfrag_toks<I: IdentFrag>(
        &mut self,
        ident: &I,
    ) -> Result<(), I::BadIdent> {
        self.with_tokens(
            |out| ident.frag_to_tokens(out), //
        )
        .unwrap_or(Ok(()))
    }
    fn append_idpath<A, B, I>(
        &mut self,
        _te_span: Span,
        pre: A,
        ident: &I,
        post: B,
        grouping: Grouping,
    ) -> Result<(), I::BadIdent>
    where
        A: FnOnce(&mut TokenAccumulator),
        B: FnOnce(&mut TokenAccumulator),
        I: IdentFrag,
    {
        let inner = match self.with_tokens(|_outer| {
            let mut inner = TokenAccumulator::new();
            pre(&mut inner);
            inner.append_identfrag_toks(ident)?;
            post(&mut inner);
            Ok(inner)
        }) {
            None => return Ok(()), // earlier errors, didn't process
            Some(Err(e)) => return Err(e),
            Some(Ok(ta)) => ta,
        };

        match inner.tokens() {
            Ok(ts) => self.append(grouping.surround(ts)),
            Err(e) => self.record_error(e),
        }
        Ok(())
    }
    fn append_syn_litstr(&mut self, lit: &syn::LitStr) {
        self.append(lit);
    }
    fn append_syn_type_inner(
        &mut self,
        _te_span: Span,
        ty: syn::Type,
        grouping: Grouping,
    ) {
        self.append(grouping.surround(ty));
    }
    fn append_tokens_with(
        &mut self,
        _not_in_paste: &(),
        f: impl FnOnce(&mut TokenAccumulator) -> syn::Result<()>,
    ) -> syn::Result<()> {
        f(self)
    }

    fn append_bool_only(&mut self, bool_only: &Self::BoolOnly) -> ! {
        void::unreachable(*bool_only)
    }

    fn record_error(&mut self, err: syn::Error) {
        if let Err(before) = &mut self.0 {
            before.combine(err);
        } else {
            self.0 = Err(err)
        }
    }

    fn new_with_span(_kw_span: Span) -> Self {
        Self::new()
    }

    fn default_subst_meta_as(kw: Span) -> syn::Result<meta::SubstAs<Self>> {
        Err(kw.error("missing `as ...` in meta expansion"))
    }

    fn ignore_impl(self) -> syn::Result<()> {
        self.0.map(|_: TokenStream| ())
    }

    fn dbg_expand(
        &mut self,
        _kw_span: Span,
        ctx: &Context,
        msg: &mut String,
        content: &Template<TokenAccumulator>,
    ) {
        let mut child = TokenAccumulator::new();
        content.expand(ctx, &mut child);
        let child = child.tokens();
        match &child {
            Err(e) => write!(msg, "/* ERROR: {} */", e),
            Ok(y) => write!(msg, "{}", y),
        }
        .expect("write! failed");
        match child {
            Ok(y) => self.append(y),
            Err(e) => self.record_error(e),
        }
    }
}
