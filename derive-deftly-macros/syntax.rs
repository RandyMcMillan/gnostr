//! Template syntax
//!
//! This module contains:
//!
//!  * The types representing a parsed template.
//!  * The `parse` methods.

use super::framework::*;

pub use SubstDetails as SD;
pub use TemplateElement as TE;

#[derive(Debug)]
pub struct Template<O: SubstParseContext> {
    /// `TD::Subst` is known not to contain `SD::when`
    pub elements: Vec<TemplateElement<O>>,
}

#[derive(Debug)]
pub struct TopTemplate {
    pub content: Template<TokenAccumulator>,
}

/// `ARG` as per
/// "Named and positional template arguments"
/// in the reference.
///
/// Parses from one of
///   *  identifier
///   *  literal
///   *  `$EXPN`
///   *  `${EXPN..}`
///   *  `{ TEMPLATE }`
#[derive(Debug)]
pub struct Argument<O: SubstParseContext = TokenAccumulator>(pub Template<O>);

#[derive(Debug)]
pub struct TemplateWithWhens<O: SubstParseContext> {
    pub elements: Vec<TemplateElement<O>>,
}

#[derive(Debug)]
pub enum TemplateElement<O: SubstParseContext> {
    Ident(Ident),
    LitStr(syn::LitStr),
    Literal(syn::Lit, O::NotInPaste),
    Punct(Punct, O::NotInPaste),
    Group {
        /// Sadly Group's constructors let us only set *both* delimiters
        delim_span: Span,
        delimiter: Delimiter,
        template: Template<O>,
        not_in_paste: O::NotInPaste,
    },
    /// Might contain `SD::when`
    Subst(Subst<O>),
    Repeat(RepeatedTemplate<O>),
}

#[derive(Debug)]
pub struct RepeatedTemplate<O: SubstParseContext> {
    pub template: Template<O>,
    #[allow(clippy::vec_box)]
    pub whens: Vec<Box<Subst<BooleanContext>>>,
    pub over: RepeatOver,
}

#[derive(Debug)]
pub struct Subst<O: SubstParseContext> {
    pub kw_span: Span,
    /// Might contain `SD::when`
    pub sd: SubstDetails<O>,
}

/// A condition, as found in `if`.
//
// Giving this a separate name is nicer but also helps avoid using
// `Subst` directly in template syntax tree nodes, which is best
// avoided because Subst<O> might contain `${when }` inappropriately.
pub type Condition = Subst<BooleanContext>;

/// Enum representing nature and payload of a substitution
///
/// This is a single enum, for all of the different lexical contexts
/// (principal expansion, identifier pasting, boolean predicates)
/// because this:
///   * Unifies the parsing code, ensuring that all these
///     constructs are parsed the same everywhere
///     (unless we deviate deliberately).
///   * Mostly unifies the expansion and loop/conditional walking code.
///   * Avoids the need to recapitulate the keywords in multiple
///     enums, or contexts with inner nested enums, or something.
///
/// The enum is generic over the lexical context [`SubstParseContext`].
/// This allows the variants that are inapplicable in a particular
/// lexical context to be made uninhabited:
/// that ensures that detection of such errors occurs during template parsing.
#[allow(non_camel_case_types)] // clearer to use the exact ident
#[derive(Debug)]
pub enum SubstDetails<O: SubstParseContext> {
    // variables
    tname(O::NotInBool),
    ttype(O::NotInBool),
    tdeftype(O::NotInBool),
    vname(O::NotInBool),
    fname(O::NotInBool),
    ftype(O::NotInBool),
    fpatname(O::NotInBool),
    Vis(SubstVis, O::NotInPaste), // tvis, fvis
    tdefkwd(O::NotInBool),

    // attributes
    Xmeta(meta::SubstMeta<O>),
    tattrs(RawAttr, O::NotInPaste, O::NotInBool),
    vattrs(RawAttr, O::NotInPaste, O::NotInBool),
    fattrs(RawAttr, O::NotInPaste, O::NotInBool),

    // generics
    tgens(O::NotInPaste),
    tdefgens(O::NotInPaste, O::NotInBool),
    tgnames(O::NotInPaste, O::NotInBool),
    twheres(O::NotInPaste, O::NotInBool),

    vpat(SubstVPat, O::NotInPaste, O::NotInBool),
    vtype(SubstVType, O::NotInPaste, O::NotInBool),

    tdefvariants(Template<TokenAccumulator>, O::NotInPaste, O::NotInBool),
    fdefine(Option<Argument>, O::NotInPaste, O::NotInBool),
    vdefbody(
        Argument<O>,
        Template<TokenAccumulator>,
        O::NotInPaste,
        O::NotInBool,
    ),

    // expansion manipulation
    paste(Template<paste::Items>, O::NotInBool),
    ChangeCase(Template<paste::Items>, paste::ChangeCase, O::NotInBool),

    // special
    when(Box<Condition>, O::NotInBool),
    define(Definition<DefinitionBody>, O::NotInBool),
    defcond(Definition<DefCondBody>, O::NotInBool),
    UserDefined(DefinitionName),

    // expressions
    False(O::BoolOnly),
    True(O::BoolOnly),
    not(Box<Condition>, O::BoolOnly),
    any(Punctuated<Condition, token::Comma>, O::BoolOnly),
    all(Punctuated<Condition, token::Comma>, O::BoolOnly),
    is_struct(O::BoolOnly),
    is_enum(O::BoolOnly),
    is_union(O::BoolOnly),
    v_is_unit(O::BoolOnly),
    v_is_tuple(O::BoolOnly),
    v_is_named(O::BoolOnly),
    is_empty(O::BoolOnly, Template<TokenAccumulator>),
    approx_equal(O::BoolOnly, [Argument; 2]),

    // Explicit iteration
    For(RepeatedTemplate<O>, O::NotInBool),
    // Conditional substitution.
    If(SubstIf<O>, O::NotInBool),
    select1(SubstIf<O>, O::NotInBool),

    ignore(Template<O>, O::NotInBool),
    error(ExplicitError, O::NotInBool),
    dbg(DbgDumpRequest<O>),
    dbg_all_keywords(O::NotInBool),

    Crate(O::NotInPaste, O::NotInBool),
}

#[derive(Debug)]
pub struct Definition<B> {
    pub name: DefinitionName,
    pub body_span: Span,
    pub body: B,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DefinitionName(syn::Ident);

#[derive(Debug)]
pub enum DefinitionBody {
    Normal(Argument),
    Paste(Argument<paste::Items>),
}

pub type DefCondBody = Box<Condition>;

#[derive(Debug)]
pub struct ExplicitError {
    pub message: syn::LitStr,
}

#[derive(Debug)]
pub struct SubstIf<O: SubstParseContext> {
    /// A series of test/result pairs.
    ///
    /// The test that gives "true"
    /// short-circuits the rest.
    pub tests: Vec<(Condition, Template<O>)>,
    /// A final element to expand if all tests fail.
    pub otherwise: Option<Box<Template<O>>>,
    pub kw_span: Span,
}

/// Whether this is `${tvis}` or `${fvis}`
#[derive(Debug)]
pub enum SubstVis {
    T,
    F,
    FD,
}

#[derive(Debug)]
pub struct SubstVType {
    pub self_: Option<Argument>,
    pub vname: Option<Argument>,
}

#[derive(Debug)]
pub struct SubstVPat {
    pub vtype: SubstVType,
    pub fprefix: Option<Argument<paste::Items>>,
}

#[derive(Debug, Clone)]
pub enum RawAttr {
    Default,
    Include {
        entries: Punctuated<RawAttrEntry, token::Comma>,
    },
    Exclude {
        exclusions: Punctuated<syn::Path, token::Comma>,
    },
}

#[derive(Debug, Clone)]
pub struct RawAttrEntry {
    pub path: syn::Path,
}

#[derive(Debug)]
pub struct DbgDumpRequest<O: SubstParseContext> {
    pub note: Option<String>,
    pub content_parsed: Box<O::DbgContent>,

    // TODO
    // We want to print the unexpanded/unevaluated template text,
    // as part of the debugging output.
    //
    // However, all our Template TokenStreams have been $-escaped,
    // so contain `$orig_dollar`.  We could fix this by unescaping
    // the whole template on input, but it is better for perf
    // to do this only when required by an actual dbg expansion.
    //
    // To resolve this TODO:
    //   - implement a function to de-escape templates
    //   - de-escape the template before making it into a String
    //   - change the type of this variable to contain the String
    //   - resolve the compile errors at the call sites
    //     by actually printing the unexpanded content
    // Or:
    //   - impl Display for Tempplate
    //     (this is probably much more work and very intrusive)
    //
    // pub content_string: String,
    pub content_string: (),
}

/// Error returned by `DefinitionName::try_from(syn::Ident)`
pub struct InvalidDefinitionName;

impl ToTokens for DefinitionName {
    fn to_tokens(&self, out: &mut TokenStream) {
        self.0.to_tokens(out)
    }
}

impl TryFrom<syn::Ident> for DefinitionName {
    type Error = InvalidDefinitionName;
    fn try_from(ident: syn::Ident) -> Result<Self, InvalidDefinitionName> {
        // We allow any identifier except those which start with a
        // lowercase letter or `_`.  Lowercase letters, and `_`, are
        // reserved for builtin functionality.  We don't restrict the
        // exclusion to *ascii* lowercase letters, even though we
        // don't intend any non-ascii keywords, because that might be
        // confusing.  proc_macros receive identifiers in NFC, so
        // we don't need to worry about whether this rule might depend
        // on the input representation.
        let s = ident.to_string();
        let c = s.chars().next().expect("identifer was empty string!");
        if c.is_lowercase() {
            Err(InvalidDefinitionName)
        } else {
            Ok(DefinitionName(ident))
        }
    }
}

impl Display for DefinitionName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Parse for DefinitionName {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.call(Ident::parse_any)?;
        let span = ident.span();
        Ok(ident.try_into().map_err(|InvalidDefinitionName| {
            span.error(
                "invalid name for definition - may not start with lowercase",
            )
        })?)
    }
}
impl Parse for Definition<DefinitionBody> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let body_span = input.span();
        let mut body: Argument<_> = input.parse()?;

        // Is it precisely an invocation of ${paste } or $< > ?
        let body = match (|| {
            if body.elements.len() != 1 {
                return None;
            }
            let Template { elements } = &mut body.0;
            // This is tedious.  We really want to move match on a vec.
            match elements.pop().expect("just checked length") {
                TE::Subst(Subst {
                    kw_span,
                    sd: SD::paste(items, not_in_bool),
                }) => Some(Argument(Template {
                    elements: vec![TE::Subst(Subst {
                        kw_span,
                        sd: SD::paste(items, not_in_bool),
                    })],
                })),
                other => {
                    // Oops, put it back
                    elements.push(other);
                    None
                }
            }
        })() {
            Some(paste) => DefinitionBody::Paste(paste),
            None => DefinitionBody::Normal(body),
        };

        Ok(Definition {
            name,
            body_span,
            body,
        })
    }
}
impl Parse for Definition<DefCondBody> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let body_span = input.span();
        let body = Box::new(input.parse()?);
        Ok(Definition {
            name,
            body_span,
            body,
        })
    }
}

impl<O: SubstParseContext> Spanned for Subst<O> {
    fn span(&self) -> Span {
        self.kw_span
    }
}

impl TopTemplate {
    pub fn parse(
        input: ParseStream,
        beta_enabled: Option<beta::Enabled>,
    ) -> syn::Result<Self> {
        let content = beta::with_maybe_enabled(
            //
            beta_enabled,
            || input.parse(),
        )?;
        Ok(TopTemplate { content })
    }
}

impl Deref for TopTemplate {
    type Target = Template<TokenAccumulator>;
    fn deref(&self) -> &Template<TokenAccumulator> {
        &self.content
    }
}

impl<O: SubstParseContext> Parse for Template<O> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Template::parse_special(input, &mut Default::default())
    }
}

impl<O: SubstParseContext> Template<O> {
    fn parse_special(
        input: ParseStream,
        special: &mut O::SpecialParseContext,
    ) -> syn::Result<Self> {
        TemplateWithWhens::parse_special(input, special)?.try_into()
    }
}

impl<O: SubstParseContext> TemplateWithWhens<O> {
    fn parse_special(
        input: ParseStream,
        mut special: &mut O::SpecialParseContext,
    ) -> syn::Result<Self> {
        // eprintln!("@@@@@@@@@@ PARSE {}", &input);
        let mut good = vec![];
        let mut errors = ErrorAccumulator::default();
        while !input.is_empty() {
            let special = &mut special;
            match errors.handle_in(|| {
                O::special_before_element_hook(special, input)
            }) {
                Some(None) => {},
                None | // error! quit parsing
                Some(Some(SpecialInstructions::EndOfTemplate)) => break,
            }
            errors.handle_in(|| {
                let elem = input.parse()?;
                good.push(elem);
                Ok(special)
            });
        }
        errors.finish_with(TemplateWithWhens { elements: good })
    }
}

impl<O: SubstParseContext> Parse for Argument<O> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let la = input.lookahead1();
        if la.peek(token::Brace) {
            let inner;
            let _brace = braced!(inner in input);
            Template::parse(&inner)
        } else if la.peek(Ident::peek_any)
            || la.peek(Token![$])
            || la.peek(syn::Lit)
        {
            let element = input.parse()?;
            TemplateWithWhens {
                elements: vec![element],
            }
            .try_into()
        } else {
            Err(la.error())
        }
        .map(Argument)
    }
}

impl<O: SubstParseContext> Deref for Argument<O> {
    type Target = Template<O>;
    fn deref(&self) -> &Template<O> {
        &self.0
    }
}

/// Proof token that [`deescape_orig_dollar`] was called
///
/// Demanded by `Subst::parse_after_dollar`.
///
/// Do not construct this yourself.
/// Only `deescape_orig_dollar` is allowed to do that.
pub enum OrigDollarDeescaped {
    /// We didn't find any `orig_dollar`
    NotFound,
    /// We *did* find `orig_dollar`; if we now see *another* `$`
    /// things get even stranger.
    Found,
}

/// Skip over any `orig_dollar`
///
/// Call this after seeing a `$`.
/// The `orig_dollar` (hopefully) came from
/// [`definition::escape_dollars`](escape_dollars).
pub fn deescape_orig_dollar(
    input: ParseStream,
) -> syn::Result<OrigDollarDeescaped> {
    input.step(|cursor| {
        let (found, rest) = (|| {
            let (ident, rest) = cursor.ident()?;
            (ident == "orig_dollar").then(|| ())?;
            Some((OrigDollarDeescaped::Found, rest))
        })()
        .unwrap_or((OrigDollarDeescaped::NotFound, *cursor));
        Ok((found, rest))
    })
}

impl<O: SubstParseContext> Parse for TemplateElement<O> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let input_span = input.span();
        let not_in_paste = || O::not_in_paste(&input_span);

        let backtracked = input.fork();

        Ok(match input.parse()? {
            TT::Group(group) => {
                let delim_span = group.span_open();
                let delimiter = group.delimiter();
                let t_parser = |input: ParseStream| Template::parse(input);
                let template = t_parser.parse2(group.stream())?;
                TE::Group {
                    not_in_paste: not_in_paste()?,
                    delim_span,
                    delimiter,
                    template,
                }
            }
            TT::Ident(tt) => TE::Ident(tt),
            tt @ TT::Literal(..) => match syn::parse2(tt.into())? {
                syn::Lit::Str(s) => TE::LitStr(s),
                other => TE::Literal(other, not_in_paste()?),
            },
            TT::Punct(tok) if tok.as_char() == '#' => {
                if let Ok(attr) = syn::Attribute::parse_inner(&backtracked) {
                    if let Some(attr) = attr.first() {
                        return Err(attr.error(
    "inner attributes are reserved syntax, anywhere in derive-deftly templates"
                        ));
                    }
                }
                TE::Punct(tok, not_in_paste()?)
            }
            TT::Punct(tok) if tok.as_char() != '$' => {
                TE::Punct(tok, not_in_paste()?)
            }
            TT::Punct(_dollar) => {
                let deescaped = deescape_orig_dollar(input)?;
                let la = input.lookahead1();
                if la.peek(Token![$]) {
                    // $$
                    let dollar: Punct = input.parse()?;
                    match deescaped {
                        OrigDollarDeescaped::NotFound => {},
                        OrigDollarDeescaped::Found => {
                            match deescape_orig_dollar(input)? {
                                OrigDollarDeescaped::Found => {},
                                OrigDollarDeescaped::NotFound => {
                                    return Err(dollar.error(
 "found `$orig_dollar $` not followed by another orig_dollar!"
                                    ))
                                },
                            }
                        },
                    }
                    TE::Punct(dollar, not_in_paste()?)
                } else if la.peek(token::Paren) {
                    RepeatedTemplate::parse_in_parens(input)?
                } else {
                    TE::Subst(Subst::parse_after_dollar(la, input, deescaped)?)
                }
            }
        })
    }
}

/// Parse `Self` using named subkeyword arguments within the `${...}`
///
/// Provides [`parse()`](ParseUsingSubkeywords::parse) in terms of:
///  * An implementation of
///    [`new_default()`](ParseUsingSubkeywords::new_default)
///    (which must generally be manually provided)
///  * An implementation of [`ParseOneSubkeyword`],
///    usually made with `impl_parse_subkeywords!`.
pub trait ParseUsingSubkeywords: Sized + ParseOneSubkeyword {
    /// Parse the body of `Self`, processing names subkeyword arguments
    ///
    /// `kw_span` is for the top-level keyword introducing `Self`.
    fn parse(input: ParseStream, kw_span: Span) -> syn::Result<Self> {
        let mut out = Self::new_default(kw_span)?;
        while !input.is_empty() {
            let subkw: IdentAny = input.parse()?;
            let _: Token![=] = input.parse()?;
            out.process_one_keyword(&subkw, input).unwrap_or_else(|| {
                Err(subkw.error("unknown $vpat/$vconstr argument sub-keyword"))
            })?;
        }
        Ok(out)
    }

    /// Make a new `Self` with default values for all parameters
    ///
    /// This is used when the keyword is invoked without being enclosed
    /// in `${...}`, and as the starting point when it *is* enclosed.
    ///
    /// `kw_span` is to be used if to construct any `SubstParseContext`
    /// lexical context tokens (eg, `NotInPaste`) in `Self`.
    fn new_default(kw_span: Span) -> syn::Result<Self>;
}

pub trait ParseOneSubkeyword: Sized {
    /// Process the value for a keyword `subkw`.
    ///
    /// `input` is inside the `{ }`, just after the `=`.
    ///
    /// Generally implemented by `impl_parse_subkeywords!`,
    /// which generates code involving a a call to [`subkw_parse_store`].
    fn process_one_keyword(
        &mut self,
        kw: &syn::Ident,
        input: ParseStream,
    ) -> Option<syn::Result<()>>;
}

/// Helper for `impl_parse_subkeywords!`; parse and store subkw argument
///
/// Parses and stores a template element argument to a subkeyword,
/// using [`Argument::parse`].
///
/// Detects repeated specification of the same keyword, as an error.
///
/// `KO` is the lexical parsing context, and determines what
/// kind of values the template author can supply.
fn subkw_parse_store<KO>(
    subkw: &syn::Ident,
    input: ParseStream,
    dest: &mut Option<Argument<KO>>,
) -> syn::Result<()>
where
    KO: SubstParseContext,
{
    if let Some(_) = &dest {
        // TODO preserve previous keyword so we can report it?
        return Err(
            subkw.error("same argument sub-keyword specified more than once")
        );
    }
    *dest = Some(input.parse()?);
    Ok(())
}

/// Implements `ParseOneSubkeyword` for the use of `ParseUsingSubkeywords`
///
/// Input syntax is `TYPE: (SUBKEYWORD-SPEC), (SUBKEYWORD-SPEC), ...`
/// where each SUBKEYWORD-SPEC is one of:
///  * `(field)`: recognises `"field"` and stores in `self.field`.
///  * `("subkw": .field)`: recognises `"subkw"`
///  * `(..substruct)`: calls `self.substruct.process_one_keyword`,
///     thereby incorporating the sub-structure's subkeywords
///
/// You can write `TYPE<O>: ...`
/// which results in
/// `impl<O: SubstParseContext> ... for TYPE<O>`.
macro_rules! impl_parse_one_subkeyword { {
    $ty:ident $( < $O:ident > )?:
    $( ( $($spec:tt)+ ) ),* $(,)?
} => {
    impl $(<$O: SubstParseContext>)?
    ParseOneSubkeyword for $ty $(<$O>)? {
        fn process_one_keyword(&mut self, got: &syn::Ident, ps: ParseStream)
                               -> Option<syn::Result<()>> {
            $( impl_parse_one_subkeyword!{ @ (self, got, ps) @ $($spec)+ } )*
            None
        }
    }
// Internal input syntax    @ (self,got,ps) @ SUBKEYWORD-SPEC
// (we must pass (self,got,ps) explicitly for annoying hygiene reasons)
}; { @ $bind:tt @ $exp:ident } => {
    impl_parse_one_subkeyword! { @@ $bind @ stringify!($exp), . $exp }
}; { @ $bind:tt @ $exp:literal: . $($field:tt)+ } => {
    impl_parse_one_subkeyword! { @@ $bind @ $exp, . $($field)+ }
}; { @ ($self:expr, $got:expr, $ps:expr) @ .. $($substruct:tt)+ } => {
    if let Some(r) = $self.$($substruct)+.process_one_keyword($got, $ps) {
        return Some(r);
    }
// Internal input syntax    @@ (self,got,ps) @ SUBKW, .FIELD-ACCESSORS
}; { @@ ($self:expr, $got:expr, $ps:expr) @ $exp:expr, .$($field:tt)+ } => {
    if $got == $exp {
        return Some(subkw_parse_store($got, $ps, &mut $self.$($field)+));
    }
} }

impl_parse_one_subkeyword! {
    SubstVType:
    ("self": .self_),
    (vname),
}

impl_parse_one_subkeyword! {
    SubstVPat:
    (..vtype),
    (fprefix),
}

impl ParseUsingSubkeywords for SubstVType {
    fn new_default(_tspan: Span) -> syn::Result<Self> {
        Ok(SubstVType {
            self_: None,
            vname: None,
        })
    }
}
impl ParseUsingSubkeywords for SubstVPat {
    fn new_default(tspan: Span) -> syn::Result<Self> {
        Ok(SubstVPat {
            vtype: SubstVType::new_default(tspan)?,
            fprefix: None,
        })
    }
}

impl<O: SubstParseContext> Subst<O> {
    /// Parses everything including a `$` (which we insist on)
    #[allow(dead_code)] // This was once used for ${paste }
    fn parse_entire(input: ParseStream) -> syn::Result<Self> {
        let _dollar: Token![$] = input.parse()?;
        let deescaped = deescape_orig_dollar(input)?;
        let la = input.lookahead1();
        Self::parse_after_dollar(la, input, deescaped)
    }

    /// Parses everything after the `$`, possibly including a pair of `{ }`
    ///
    /// You must have called [`deescape_orig_dollar`],
    /// and handled the `$$` case.
    fn parse_after_dollar(
        la: Lookahead1,
        input: ParseStream,
        _deescaped: OrigDollarDeescaped,
    ) -> syn::Result<Self> {
        if la.peek(token::Brace) {
            let exp;
            struct Only<O: SubstParseContext>(Subst<O>);
            impl<O: SubstParseContext> Parse for Only<O> {
                fn parse(input: ParseStream) -> syn::Result<Self> {
                    let subst = input.parse()?;
                    let unwanted: Option<TT> = input.parse()?;
                    if let Some(unwanted) = unwanted {
                        return Err(unwanted.error(
                            "unexpected arguments to expansion keyword",
                        ));
                    }
                    Ok(Only(subst))
                }
            }
            let _brace = braced!(exp in input);
            let exp = exp.parse()?;
            let Only(exp) = exp;
            Ok(exp)
        } else if la.peek(syn::Ident::peek_any) {
            let exp: TokenTree = input.parse()?; // get it as TT
            let exp = syn::parse2(exp.to_token_stream())?;
            Ok(exp)
        } else if la.peek(Token![<]) {
            let angle: Token![<] = input.parse()?;
            let state = paste::AngleBrackets::default();
            let mut special = Some(state);
            let template = Template::parse_special(input, &mut special)?;
            let state = special.unwrap();
            state.finish(angle.span())?;
            Ok(Subst {
                kw_span: angle.span(),
                sd: SD::paste(template, O::not_in_bool(&angle)?),
            })
        } else {
            return Err(la.error());
        }
    }
}

/// Parses only the content (ie, after the `$` and inside any `{ }`)
impl<O: SubstParseContext> Parse for Subst<O> {
    fn parse<'i>(input: ParseStream<'i>) -> syn::Result<Self> {
        let kw: IdentAny = input.parse()?;
        let from_sd = |sd| {
            Ok(Subst {
                sd,
                kw_span: kw.span(),
            })
        };

        // See `tests/pub-export/pub-b/pub-b.rs`
        #[cfg(feature = "bizarre")]
        let kw = {
            let s = kw.to_string();
            let s = s
                .strip_suffix("_bizarre")
                .ok_or_else(|| kw.error("bizarre mode but not _bizarre"))?;
            IdentAny(syn::Ident::new(s, kw.span()))
        };

        // keyword!{ KEYWORD [ {BLOCK WITH BINDINGS} ] [ CONSTRUCTOR-ARGS ] }
        // expands to something like:
        //
        //   if supplied_keyword = "KEYWORD" {
        //       return Ok(Subst {
        //           sd: SubstDetails::KEYWORD CONSTRUCTOR-ARGS,
        //           ..
        //       })
        //   }
        //
        // KEYWORD can be "KEYWORD_STRING": CONSTRUCTOR,
        // in case the enum variant name is not precisely the keyword.
        //
        // See `keyword_general!` in utils.rs for full details.
        macro_rules! keyword { { $($args:tt)* } => {
            keyword_general! { kw from_sd SD; $($args)* }
        } }

        let not_in_paste = || O::not_in_paste(&kw);
        let not_in_bool = || O::not_in_bool(&kw);
        let bool_only = || O::bool_only(&kw);

        let parse_if = |input| SubstIf::parse(input, kw.span());

        let in_parens = |input: ParseStream<'i>| {
            let inner;
            let _paren = parenthesized!(inner in input);
            Ok(inner)
        };

        let parse_def_body = |input: ParseStream<'i>, m| {
            if input.is_empty() {
                return Err(kw.error(m));
            }
            Template::parse(input)
        };

        let parse_meta =
            |input, scope| meta::SubstMeta::parse(input, kw.span(), scope);

        keyword! { tname(not_in_bool()?) }
        keyword! { ttype(not_in_bool()?) }
        keyword! { tdeftype(not_in_bool()?) }
        keyword! { vname(not_in_bool()?) }
        keyword! { fname(not_in_bool()?) }
        keyword! { ftype(not_in_bool()?) }
        keyword! { fpatname(not_in_bool()?) }
        keyword! { tdefkwd(not_in_bool()?) }

        keyword! { "tvis": Vis(SubstVis::T, not_in_paste()?) }
        keyword! { "fvis": Vis(SubstVis::F, not_in_paste()?) }
        keyword! { "fdefvis": Vis(SubstVis::FD, not_in_paste()?) }

        keyword! { is_struct(bool_only()?) }
        keyword! { is_enum(bool_only()?) }
        keyword! { is_union(bool_only()?) }
        keyword! { v_is_unit(bool_only()?) }
        keyword! { v_is_tuple(bool_only()?) }
        keyword! { v_is_named(bool_only()?) }

        keyword! { tgens(not_in_paste()?) }
        keyword! { tdefgens(not_in_paste()?, not_in_bool()?) }
        keyword! { tgnames(not_in_paste()?, not_in_bool()?) }
        keyword! { twheres(not_in_paste()?, not_in_bool()?) }

        use meta::Scope as MS;
        keyword! { "tmeta": Xmeta(parse_meta(input, MS::T)?) }
        keyword! { "vmeta": Xmeta(parse_meta(input, MS::V)?) }
        keyword! { "fmeta": Xmeta(parse_meta(input, MS::F)?) }

        keyword! { tattrs(input.parse()?, not_in_paste()?, not_in_bool()?) }
        keyword! { vattrs(input.parse()?, not_in_paste()?, not_in_bool()?) }
        keyword! { fattrs(input.parse()?, not_in_paste()?, not_in_bool()?) }

        keyword! { vtype(
            SubstVType::parse(input, kw.span())?,
            not_in_paste()?, not_in_bool()?,
        ) }
        keyword! { vpat(
            SubstVPat::parse(input, kw.span())?,
            not_in_paste()?, not_in_bool()?,
        ) }

        keyword! { tdefvariants(
            parse_def_body(
                input,
                "tdefvariants needs to contain the variant definitions",
            )?,
            not_in_paste()?, not_in_bool()?,
        ) }
        keyword! { fdefine(
            (!input.is_empty()).then(|| input.parse()).transpose()?,
            not_in_paste()?, not_in_bool()?
        ) }
        keyword! { vdefbody(
            input.parse()?,
            parse_def_body(
                input,
                "vdefbody needs to contain the body definition",
            )?,
            not_in_paste()?, not_in_bool()?,
        ) }
        keyword! { is_empty(bool_only()?, {
            let content;
            let _ = parenthesized!(content in input);
            content.parse()?
        }) }
        keyword! { approx_equal(bool_only()?, {
            let args =
                Punctuated::<_, Token![,]>::parse_separated_nonempty(
                    &in_parens(input)?,
                )?;
            if args.len() != 2 {
                return Err(kw.error(
                    "approx_equal() requires two comma-separated arguments"
                ))
            }
            let mut args = args.into_iter();
            // std::array::from_fn needs MSRV 1.63
            let mut arg = || args.next().unwrap();
            [ arg(), arg() ]
        }) }

        keyword! { paste(Template::parse(input)?, not_in_bool()?) }
        keyword! { when(input.parse()?, not_in_bool()?) }
        keyword! { define(input.parse()?, not_in_bool()?) }
        keyword! { defcond(input.parse()?, not_in_bool()?) }

        keyword! { "false": False(bool_only()?) }
        keyword! { "true": True(bool_only()?) }
        keyword! { "if": If(parse_if(input)?, not_in_bool()?) }
        keyword! { select1(parse_if(input)?, not_in_bool()?) }
        keyword! { ignore(input.parse()?, not_in_bool()?) }
        keyword! { error(input.parse()?, not_in_bool()?) }
        keyword! { dbg(input.parse()?) }
        keyword! { dbg_all_keywords(not_in_bool()?) }
        keyword! { "crate": Crate(not_in_paste()?, not_in_bool()?) }
        keyword! { "_dd_intern_crate": Crate(not_in_paste()?, not_in_bool()?) }

        keyword! { "for": For(
            RepeatedTemplate::parse_for(input)?,
            not_in_bool()?,
        )}

        let any_all_contents = |input: ParseStream<'i>| {
            Punctuated::parse_terminated(&in_parens(input)?)
        };
        keyword! { any(any_all_contents(input)?, bool_only()?) }
        keyword! { all(any_all_contents(input)?, bool_only()?) }
        keyword! { not(in_parens(input)?.parse()?, bool_only()?) }

        if let Ok(case) = kw.to_string().parse() {
            return from_sd(SD::ChangeCase(
                Template::parse(input)?,
                case,
                not_in_bool()?,
            ));
        }

        if let Ok(user_defined) = kw.clone().try_into() {
            return from_sd(SD::UserDefined(user_defined));
        }

        Err(kw.error("unknown derive-deftly keyword"))
    }
}

impl<O: SubstParseContext> Parse for DbgDumpRequest<O> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        O::parse_maybe_within_parens(input, |input| {
            let note = if input.peek(syn::LitStr) {
                let note: syn::LitStr = input.parse()?;
                O::parse_maybe_comma(input)?;
                Some(note.value())
            } else {
                None
            };

            let content_buf;
            let content = if O::IS_BOOL {
                input
            } else {
                let _ = braced!(content_buf in input);
                &content_buf
            };
            // was content.to_string(); but $orig_dollar, see TODO
            let content_string = ();
            let content_parsed = content.parse()?;
            Ok(DbgDumpRequest {
                note,
                content_string,
                content_parsed,
            })
        })
    }
}

impl<O: SubstParseContext> DbgDumpRequest<O> {
    pub fn display_heading<'s>(
        &'s self,
        ctx: &'s Context,
    ) -> impl Display + 's {
        struct Adapter<'a, O: SubstParseContext>(
            &'a DbgDumpRequest<O>,
            &'a Context<'a>,
        );

        impl<O: SubstParseContext> Display for Adapter<'_, O> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                let Adapter(ddr, ctx) = self;
                if let Some(note) = &ddr.note {
                    write!(f, "{:?} ", &note)?;
                }
                write!(f, "{}", ctx.display_for_dbg())
            }
        }

        Adapter(self, ctx)
    }
}

impl Parse for ExplicitError {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let message: syn::LitStr = input.parse()?;
        if message.suffix() != "" {
            return Err(message.error("suffix forbidden on error string"));
        }
        Ok(ExplicitError { message })
    }
}

impl<O: SubstParseContext> SubstIf<O> {
    fn parse(input: ParseStream, kw_span: Span) -> syn::Result<Self> {
        let mut tests = Vec::new();
        let mut otherwise = None;

        loop {
            let condition = input.parse()?;
            let content;
            let _br = braced![ content in input ];
            let consequence = Template::parse(&content)?;
            tests.push((condition, consequence));

            // (I'd like to use a lookahead here too, but it doesn't
            // accept "Nothing")
            if input.is_empty() {
                // no more conditions if there is not an "else"
                break;
            }

            let lookahead1 = input.lookahead1();
            if lookahead1.peek(syn::Ident) {
                // this is another expansion keyword, then
                // skipped `else if`
                continue;
            } else if lookahead1.peek(Token![else]) {
                // else clause
            } else {
                return Err(lookahead1.error());
            }

            let _else: Token![else] = input.parse()?;

            let lookahead = input.lookahead1();
            if lookahead.peek(Token![if]) {
                let _if: Token![if] = input.parse()?;
                // got an "else if": continue to the next iteration.
                continue;
            } else if lookahead.peek(token::Brace) {
                let content;
                let _br = braced![ content in input ];
                otherwise = Some(Template::parse(&content)?.into());
                break;
                // No more input allowed.
                // Subst::parse_after_dollar will detect any remaining
                // tokens and make an error if there are any.
            } else {
                return Err(lookahead.error());
            }
        }

        Ok(SubstIf {
            kw_span,
            tests,
            otherwise,
        })
    }
}

impl SubstVis {
    pub fn syn_vis<'c>(
        &self,
        ctx: &'c Context<'c>,
        tspan: Span,
    ) -> syn::Result<&'c syn::Visibility> {
        let field_decl_vis =
            || Ok::<_, syn::Error>(&ctx.field(&tspan)?.field.vis);
        Ok(match self {
            SubstVis::T => &ctx.top.vis,
            SubstVis::FD => field_decl_vis()?,
            SubstVis::F => {
                // Cause an error for fvis in enums, even though
                // we only look at the toplevel visibility.
                let field = field_decl_vis()?;
                match ctx.top.data {
                    syn::Data::Struct(_) | syn::Data::Union(_) => field,
                    syn::Data::Enum(_) => &ctx.top.vis,
                }
            }
        })
    }
}

impl RawAttrEntry {
    fn simple(self, _negated: &Token![!]) -> syn::Result<syn::Path> {
        Ok(self.path)
    }
}

impl Parse for RawAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(RawAttr::Default);
        }

        let la = input.lookahead1();
        let negated;
        if la.peek(Token![!]) {
            negated = Some(input.parse()?);
        } else if la.peek(Token![=]) {
            let _: Token![=] = input.parse()?;
            negated = None;
        } else {
            negated = None;
        }

        let entries: Punctuated<RawAttrEntry, _> =
            input.call(Punctuated::parse_terminated)?;

        if let Some(negated) = &negated {
            let exclusions = entries
                .into_iter()
                .map(|ent| ent.simple(negated))
                .try_collect()?;
            Ok(RawAttr::Exclude { exclusions })
        } else {
            Ok(RawAttr::Include { entries })
        }
    }
}

impl Parse for RawAttrEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path = input.parse()?;
        Ok(RawAttrEntry { path })
    }
}

/// Matches `$te` looking for `SD::when`
///
/// If so, returns `Left(kw_span, Box<Condition>)`.
/// Otherwise returns Right($e).
///
/// `$te: TemplateElement` or `$te: &TemplateElement`
macro_rules! te_extract_when { { $te:expr } => {
    match $te {
        TE::Subst(Subst { kw_span, sd: SD::when(bc, _) }) => {
            Left((kw_span, bc))
        }
        other => Right(other),
    }
} }

impl<O: SubstParseContext> RepeatedTemplate<O> {
    fn parse_in_parens(input: ParseStream) -> syn::Result<TemplateElement<O>> {
        let template;
        let paren = parenthesized!(template in input);
        let rt = RepeatedTemplate::parse(&template, paren.span, None)?;
        Ok(TE::Repeat(rt))
    }

    fn parse_for(input: ParseStream) -> syn::Result<RepeatedTemplate<O>> {
        let over: Ident = input.parse()?;
        let over = if over == "fields" {
            RepeatOver::Fields
        } else if over == "variants" {
            RepeatOver::Variants
        } else {
            return Err(
                over.error("$for must be followed by 'fields' or 'variants'")
            );
        };
        let template;
        let brace = braced!(template in input);
        RepeatedTemplate::parse(&template, brace.span, Some(over))
    }

    fn parse(
        input: ParseStream,
        span: DelimSpan,
        over: Option<RepeatOver>,
    ) -> Result<RepeatedTemplate<O>, syn::Error> {
        use TemplateWithWhens as TWW;

        let TWW { elements } =
            TWW::parse_special(input, &mut Default::default())?;

        // split `when` off
        let mut elements = VecDeque::from(elements);
        let mut whens = vec![];

        while let Some(()) = {
            match elements.pop_front().map(|e| te_extract_when!(e)) {
                Some(Left((_kw_span, bc))) => {
                    whens.push(bc);
                    Some(())
                }
                Some(Right(other)) => {
                    elements.push_front(other);
                    None
                }
                None => None,
            }
        } {}

        let elements = Vec::from(elements);
        let template = TemplateWithWhens { elements };
        let template = Template::try_from(template)?;

        let over = match over {
            Some(over) => Ok(over),
            None => {
                let mut visitor = RepeatAnalysisVisitor::default();
                template.analyse_repeat(&mut visitor)?;
                visitor.finish(span)
            }
        };

        match over {
            Ok(over) => Ok(RepeatedTemplate {
                over,
                template,
                whens,
            }),
            Err(errs) => Err(errs),
        }
    }
}

impl<O> TryFrom<TemplateWithWhens<O>> for Template<O>
where
    O: SubstParseContext,
{
    type Error = syn::Error;

    fn try_from(unchecked: TemplateWithWhens<O>) -> syn::Result<Template<O>> {
        let TemplateWithWhens { elements } = unchecked;
        for e in &elements {
            if let Left((kw_span, _)) = te_extract_when!(e) {
                return Err(kw_span.error(
 "${when } must be at the top-level of a repetition, before other content"
                ));
            }
        }
        Ok(Template { elements })
    }
}

pub fn preprocess_attrs(
    attrs: &[syn::Attribute],
) -> syn::Result<meta::PreprocessedMetas> {
    attrs
        .iter()
        .filter_map(|attr| {
            // infallible filtering for attributes we are interested in
            match attr.style {
                syn::AttrStyle::Outer => {}
                syn::AttrStyle::Inner(_) => return None,
            };
            if attr.path().leading_colon.is_some() {
                return None;
            }
            let segment = attr.path().segments.iter().exactly_one().ok()?;
            if segment.ident != "deftly" {
                return None;
            }
            Some(attr)
        })
        .map(|attr| {
            attr.call_in_parens(meta::PreprocessedValueList::parse_inner)
        })
        .collect()
}

pub fn preprocess_fields(
    fields: &syn::Fields,
) -> syn::Result<Vec<PreprocessedField>> {
    let fields = match fields {
        syn::Fields::Named(f) => &f.named,
        syn::Fields::Unnamed(f) => &f.unnamed,
        syn::Fields::Unit => return Ok(vec![]),
    };
    fields
        .into_iter()
        .map(|field| {
            let pmetas = preprocess_attrs(&field.attrs)?;
            Ok(PreprocessedField { pmetas })
        })
        .collect()
}
