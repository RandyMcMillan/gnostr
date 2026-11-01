//! `#[deftly(...)]` meta attributes
//!
//! # Used meta checking
//!
//! Most of this file is concerned with generating
//! accurate and useful messages
//! when a driver is decorated with `#[deftly(...)]` attributes
//! which are not used by any template.
//!
//! We distinguish "used" metas from "recognised" ones.
//!
//! "Used" ones are those actually tested, and used,
//! during the dynamic expansion of the template.
//! They are recorded in the [`PreprocessedMetas`],
//! which contains a `Cell` for each supplied node.
//!
//! "Recognised" ones are those which appear anywhere in the template.
//! These are represented in a data structure [``Recognised`].
//! This is calculated by scanning the template,
//! using the `FindRecogMetas` trait.
//!
//! Both of these sets are threaded through
//! the ACCUM data in successive template expansions;
//! in the final call (`EngineFinalInput`),
//! they are combined together,
//! and the driver's metas are checked against them.

use super::framework::*;

use indexmap::IndexMap;

use Usage as U;

//---------- common definitions ----------

/// Indicates one of `fmeta`, `vmeta` or `tmeta`
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[derive(AsRefStr, EnumString, EnumIter)]
#[rustfmt::skip]
pub enum Scope {
    // NB these keywords are duplicated in SubstDetails
    #[strum(serialize = "tmeta")] T,
    #[strum(serialize = "vmeta")] V,
    #[strum(serialize = "fmeta")] F,
}

/// Scope of a *supplied* meta (`#[deftly(...)]`) attribute
///
/// Also encodes, for metas at the toplevel,
/// whether it's a struct or an enum.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)] //
#[derive(strum::Display, EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum SuppliedScope {
    Struct,
    Enum,
    Variant,
    Field,
}

impl SuppliedScope {
    fn recog_search(self) -> impl Iterator<Item = Scope> {
        use Scope as S;
        use SuppliedScope as SS;
        match self {
            SS::Struct => &[S::T, S::V] as &[_],
            SS::Enum => &[S::T],
            SS::Variant => &[S::V],
            SS::Field => &[S::F],
        }
        .iter()
        .copied()
    }
}

/// `(foo(bar))` in eg `fmeta(foo(bar))`
///
/// includes the parens
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Label {
    // Nonempty list, each with nonempty segments.
    // Outermost first.
    pub lpaths: Vec<syn::Path>,
}

/// Meta designator eg `fmeta(foo(bar))`
// Field order must be the same as BorrowedDesig
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Desig {
    pub scope: Scope,
    pub label: Label,
}

#[derive(Hash)]
// Field order must be the same as meta::Desig
struct BorrowedDesig<'p> {
    pub scope: Scope,
    pub lpaths: &'p [&'p syn::Path],
}

//---------- substitutions in a template ----------

#[derive(Debug)]
pub struct SubstMeta<O: SubstParseContext> {
    pub desig: Desig,
    pub as_: Option<SubstAs<O>>,
    pub default: Option<(Argument<O>, O::NotInBool, beta::Enabled)>,
}

#[derive(Debug, Clone, AsRefStr, Display)]
#[allow(non_camel_case_types)] // clearer to use the exact ident
pub enum SubstAs<O: SubstParseContext> {
    expr(O::NotInBool, O::NotInPaste, SubstAsSupported<ValueExpr>),
    ident(O::NotInBool),
    items(O::NotInBool, O::NotInPaste, SubstAsSupported<ValueItems>),
    path(O::NotInBool),
    str(O::NotInBool),
    token_stream(O::NotInBool, O::NotInPaste),
    ty(O::NotInBool),
}

//---------- meta attrs in a driver ----------

/// A part like `(foo,bar(baz),zonk="value")`
#[derive(Debug)]
pub struct PreprocessedValueList {
    pub content: Punctuated<PreprocessedTree, token::Comma>,
}

/// `#[deftly(...)]` helper attributes
pub type PreprocessedMetas = Vec<PreprocessedValueList>;

/// An `#[deftly()]` attribute, or a sub-tree within one
///
/// Has interior mutability, for tracking whether the value is used.
/// (So should ideally not be Clone, to help avoid aliasing bugs.)
#[derive(Debug)]
pub struct PreprocessedTree {
    pub path: syn::Path,
    pub value: PreprocessedValue,
    pub used: Cell<Option<Usage>>,
}

/// Content of a meta attribute
///
/// Examples in doc comments are for
/// `PreprocessedMeta.path` of `foo`,
/// ie the examples are for `#[deftly(foo ..)]`.
#[derive(Debug)]
pub enum PreprocessedValue {
    /// `#[deftly(foo)]`
    Unit,
    /// `#[deftly(foo = "lit")]`
    Value { value: syn::Lit },
    /// `#[deftly(foo(...))]`
    List(PreprocessedValueList),
}

//---------- search and match results ----------

/// Node in tree structure found in driver `#[deftly(some(thing))]`
#[derive(Debug)]
pub struct FoundNode<'l> {
    kind: FoundNodeKind<'l>,
    path_span: Span,
    ptree: &'l PreprocessedTree,
}

/// Node in tree structure found in driver `#[deftly(some(thing))]`
#[derive(Debug)]
pub enum FoundNodeKind<'l> {
    Unit,
    Lit(&'l syn::Lit),
}

/// Information about a nearby meta node we found
///
/// "Nearby" means that the node we found is a prefix (in tree descent)
/// of the one we were looking for, or vice versa.
#[derive(Debug)]
pub struct FoundNearbyNode<'l> {
    pub kind: FoundNearbyNodeKind,
    /// Span of the identifier in the actual `#[deftly]` driver attribute
    pub path_span: Span,
    pub ptree: &'l PreprocessedTree,
}

/// How the nearby node relates to the one we were looking for
#[derive(Debug)]
pub enum FoundNearbyNodeKind {
    /// We were looking to go deeper, but found a unit in `#[deftly]`
    Unit,
    /// We were looking to go deeper, but found a `name = value` in `#[deftly]`
    Lit,
    /// We were looking for a leaf, but we found nested list in `#[deftly]`
    List,
}

pub use FoundNearbyNodeKind as FNNK;
pub use FoundNodeKind as FNK;

//---------- meta attr enumeration and checking ----------

/// Whether a meta node was used (or ought to be used)
#[derive(Debug, Copy, Clone, Hash, PartialOrd, Ord, Eq, PartialEq)] //
#[derive(EnumIter)]
pub enum Usage {
    BoolOnly,
    Value,
}

/// One lot of used metas in accumulation - argument to a `_meta_used` accum
#[derive(Debug)]
pub struct UsedGroup {
    pub content: TokenStream,
}

/// Something representing possibly checking that meta attributes are used
#[derive(Debug, Clone)]
pub enum CheckUsed<T> {
    /// Yes, check them, by/with/from/to `T`
    Check(T),
    /// No, don't check them.
    Unchecked,
}

/// Information for meta checking, found in accumulation
#[derive(Debug, Default)]
pub struct Accum {
    pub recog: Recognised,
    pub used: Vec<UsedGroup>,
}

#[derive(Default, Debug, Clone)]
pub struct Recognised {
    map: IndexMap<Desig, Usage>,
}

pub trait FindRecogMetas {
    /// Search for `fmeta(..)` etc. expansions
    ///
    /// Add to `acc` any that are
    /// (recusively) within `self`, syntactically,
    fn find_recog_metas(&self, acc: &mut Recognised);
}

//==================== implementations) ====================

//---------- template parsing ----------

impl<O: SubstParseContext> SubstMeta<O> {
    fn span_whole(&self, scope_span: Span) -> Span {
        spans_join(chain!(
            [scope_span], //
            self.desig.label.spans(),
        ))
        .unwrap()
    }
}

impl Label {
    /// Nonempty
    pub fn spans(&self) -> impl Iterator<Item = Span> + '_ {
        self.lpaths.iter().map(|path| path.span())
    }
}

impl<O: SubstParseContext> SubstAs<O> {
    fn parse(input: ParseStream, nb: O::NotInBool) -> syn::Result<Self> {
        let kw: IdentAny = input.parse()?;
        let from_sma = |sma: SubstAs<_>| Ok(sma);

        // See keyword_general! in utils.rs
        macro_rules! keyword { { $($args:tt)* } => {
            keyword_general! { kw from_sma SubstAs; $($args)* }
        } }

        let not_in_paste = || O::not_in_paste(&kw);
        fn supported<P>(kw: &IdentAny) -> syn::Result<SubstAsSupported<P>>
        where
            P: SubstAsSupportStatus,
        {
            SubstAsSupportStatus::new(&kw)
        }

        keyword! { expr(nb, not_in_paste()?, supported(&kw)?) }
        keyword! { ident(nb) }
        keyword! { items(nb, not_in_paste()?, supported(&kw)?) }
        keyword! { path(nb) }
        keyword! { str(nb) }
        keyword! { token_stream(nb, not_in_paste()?) }
        keyword! { ty(nb) }

        Err(kw.error("unknown derive-deftly 'as' syntax type keyword"))
    }
}

impl<O: SubstParseContext> SubstMeta<O> {
    pub fn parse(
        input: ParseStream,
        kw_span: Span,
        scope: Scope,
    ) -> syn::Result<Self> {
        if input.is_empty() {
            O::missing_keyword_arguments(kw_span)?;
        }

        let label: Label = input.parse()?;

        fn store<V>(
            kw: Span,
            already: &mut Option<(Span, V)>,
            call: impl FnOnce() -> syn::Result<V>,
        ) -> syn::Result<()> {
            if let Some((already, _)) = already {
                return Err([(*already, "first"), (kw, "second")]
                    .error("`${Xmeta ..}` option repeated"));
            }
            let v = call()?;
            *already = Some((kw, v));
            Ok(())
        }

        let mut as_ = None::<(Span, SubstAs<O>)>;
        let mut default = None;

        while !O::IS_BOOL && !input.is_empty() {
            let keyword = Ident::parse_any(input)?;
            let kw_span = keyword.span();
            let nb = O::not_in_bool(&kw_span).expect("checked already");
            let ue = || beta::Enabled::new_for_syntax(kw_span);
            if keyword == "as" {
                store(kw_span, &mut as_, || SubstAs::parse(input, nb))?;
            } else if keyword == "default" {
                store(kw_span, &mut default, || {
                    Ok((input.parse()?, nb, ue()?))
                })?;
            } else {
                return Err(keyword.error("unknown option in `${Xmeta }`"));
            }
            if input.is_empty() {
                break;
            }
            let _: Token![,] = input.parse()?;
        }

        macro_rules! ret { { $( $f:ident )* } => {
            SubstMeta {
                desig: Desig { label, scope },
                $( $f: $f.map(|(_span, v)| v), )*
            }
        } }

        Ok(ret! {
            as_
            default
        })
    }
}

//---------- driver parsing ----------

impl PreprocessedValueList {
    fn parse_outer(input: ParseStream) -> syn::Result<Self> {
        let meta;
        let _paren = parenthesized!(meta in input);
        Self::parse_inner(&meta)
    }
}

impl PreprocessedValueList {
    pub fn parse_inner(input: ParseStream) -> syn::Result<Self> {
        let content = Punctuated::parse_terminated(input)?;
        Ok(PreprocessedValueList { content })
    }
}

impl Parse for PreprocessedTree {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        use PreprocessedValue as PV;

        let path = input.call(syn::Path::parse_mod_style)?;
        let la = input.lookahead1();
        let value = if la.peek(Token![=]) {
            let _: Token![=] = input.parse()?;
            let value = input.parse()?;
            PV::Value { value }
        } else if la.peek(token::Paren) {
            let list = input.call(PreprocessedValueList::parse_outer)?;
            PV::List(list)
        } else if la.peek(Token![,]) || input.is_empty() {
            PV::Unit
        } else {
            return Err(la.error());
        };
        let used = None.into(); // will be filled in later
        Ok(PreprocessedTree { path, value, used })
    }
}

impl Parse for Label {
    fn parse(outer: ParseStream) -> syn::Result<Self> {
        fn recurse(
            lpaths: &mut Vec<syn::Path>,
            outer: ParseStream,
        ) -> syn::Result<()> {
            let input;
            let paren = parenthesized!(input in outer);
            let path = input.call(syn::Path::parse_mod_style)?;
            if path.segments.is_empty() {
                return Err(paren
                    .span
                    .error("`deftly` attribute must have nonempty path"));
            }

            lpaths.push(path);
            if !input.is_empty() {
                recurse(lpaths, &input)?;
            }
            Ok(())
        }

        let mut lpaths = vec![];
        recurse(&mut lpaths, outer)?;

        Ok(Label { lpaths })
    }
}

//---------- searching and matching ----------

impl Label {
    /// Caller must note meta attrs that end up being used!
    pub fn search<'a, F, G, E>(
        &self,
        pmetas: &'a [PreprocessedValueList],
        f: &mut F,
        g: &mut G,
    ) -> Result<(), E>
    where
        F: FnMut(FoundNode<'a>) -> Result<(), E>,
        G: FnMut(FoundNearbyNode<'a>) -> Result<(), E>,
    {
        for m in pmetas {
            for l in &m.content {
                Self::search_1(&self.lpaths, l, &mut *f, &mut *g)?;
            }
        }
        Ok(())
    }

    fn search_1<'a, E, F, G>(
        // Nonempty
        lpaths: &[syn::Path],
        ptree: &'a PreprocessedTree,
        f: &mut F,
        g: &mut G,
    ) -> Result<(), E>
    where
        F: FnMut(FoundNode<'a>) -> Result<(), E>,
        G: FnMut(FoundNearbyNode<'a>) -> Result<(), E>,
    {
        use PreprocessedValue as PV;

        if ptree.path != lpaths[0] {
            return Ok(());
        }

        let path_span = ptree.path.span();

        let mut nearby = |kind| {
            g(FoundNearbyNode {
                kind,
                path_span,
                ptree,
            })
        };

        let deeper = if lpaths.len() <= 1 {
            None
        } else {
            Some(&lpaths[1..])
        };

        match (deeper, &ptree.value) {
            (None, PV::Unit) => f(FoundNode {
                path_span,
                kind: FNK::Unit,
                ptree,
            })?,
            (None, PV::List(_)) => nearby(FNNK::List)?,
            (None, PV::Value { value, .. }) => f(FoundNode {
                path_span,
                kind: FNK::Lit(value),
                ptree,
            })?,
            (Some(_), PV::Value { .. }) => nearby(FNNK::Lit)?,
            (Some(_), PV::Unit) => nearby(FNNK::Unit)?,
            (Some(d), PV::List(l)) => {
                for m in l.content.iter() {
                    Self::search_1(d, m, &mut *f, &mut *g)?;
                }
            }
        }

        Ok(())
    }
}

impl Label {
    pub fn search_eval_bool(
        &self,
        pmetas: &PreprocessedMetas,
    ) -> Result<(), Found> {
        let found = |ptree: &PreprocessedTree| {
            ptree.update_used(Usage::BoolOnly);
            Err(Found)
        };
        self.search(
            pmetas,
            &mut |av| /* got it! */ found(av.ptree),
            &mut |nearby| match nearby.kind {
                FNNK::List => found(nearby.ptree),
                FNNK::Unit => Ok(()),
                FNNK::Lit => Ok(()),
            },
        )
    }
}

//---------- scope and designator handling ----------

impl<O> SubstMeta<O>
where
    O: SubstParseContext,
{
    pub fn repeat_over(&self) -> Option<RepeatOver> {
        match self.desig.scope {
            Scope::T => None,
            Scope::V => Some(RO::Variants),
            Scope::F => Some(RO::Fields),
        }
    }
}

impl<O> SubstMeta<O>
where
    O: SubstParseContext,
{
    pub fn pmetas<'c>(
        &self,
        ctx: &'c Context<'c>,
        kw_span: Span,
    ) -> syn::Result<&'c PreprocessedMetas> {
        Ok(match self.desig.scope {
            Scope::T => &ctx.pmetas,
            Scope::V => &ctx.variant(&kw_span)?.pmetas,
            Scope::F => &ctx.field(&kw_span)?.pfield.pmetas,
        })
    }
}

impl ToTokens for Label {
    fn to_tokens(&self, out: &mut TokenStream) {
        let mut lpaths = self.lpaths.iter().rev();
        let mut current =
            lpaths.next().expect("empty path!").to_token_stream();
        let r = loop {
            let span = current.span();
            let mut group =
                proc_macro2::Group::new(Delimiter::Parenthesis, current);
            group.set_span(span);
            let wrap = if let Some(y) = lpaths.next() {
                y
            } else {
                break group;
            };
            current = quote! { #wrap #group };
        };
        r.to_tokens(out);
    }
}

impl Desig {
    fn to_tokens(&self, scope_span: Span, out: &mut TokenStream) {
        let scope: &str = self.scope.as_ref();
        Ident::new(scope, scope_span).to_tokens(out);
        self.label.to_tokens(out);
    }
}

impl Parse for Desig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let scope: syn::Ident = input.parse()?;
        let scope = scope
            .to_string()
            .parse()
            .map_err(|_| scope.error("invalid meta keyword/level"))?;
        let label = input.parse()?;
        Ok(Self { scope, label })
    }
}

impl indexmap::Equivalent<Desig> for BorrowedDesig<'_> {
    fn equivalent(&self, desig: &Desig) -> bool {
        let BorrowedDesig { scope, lpaths } = self;
        *scope == desig.scope
            && itertools::equal(lpaths.iter().copied(), &desig.label.lpaths)
    }
}

/// `Display`s as a `#[deftly(...)]` as the user might write it
struct DisplayAsIfSpecified<'r> {
    lpaths: &'r [&'r syn::Path],
    /// Included after the innermost lpath, inside the parens
    inside_after: &'r str,
}
impl Display for DisplayAsIfSpecified<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#[deftly")?;
        for p in self.lpaths {
            write!(f, "({}", p.to_token_stream())?;
        }
        write!(f, "{}", self.inside_after)?;
        for _ in self.lpaths {
            write!(f, ")")?;
        }
        Ok(())
    }
}

// Tests that our `BorrowedDesig` `equivalent` impl is justified.
#[test]
fn check_borrowed_desig() {
    use super::*;
    use indexmap::Equivalent;
    use itertools::iproduct;
    use std::hash::{Hash, Hasher};

    #[derive(PartialEq, Eq, Debug, Default)]
    struct TrackingHasher(Vec<Vec<u8>>);
    impl Hasher for TrackingHasher {
        fn write(&mut self, bytes: &[u8]) {
            self.0.push(bytes.to_owned());
        }
        fn finish(&self) -> u64 {
            unreachable!()
        }
    }
    impl TrackingHasher {
        fn hash(t: impl Hash) -> Self {
            let mut self_ = Self::default();
            t.hash(&mut self_);
            self_
        }
    }

    type Case = (Scope, &'static [&'static str]);
    const TEST_CASES: &[Case] = &[
        (Scope::T, &["path"]),
        (Scope::T, &["r#path"]),
        (Scope::V, &["path", "some::path"]),
        (Scope::V, &["r#struct", "with_generics::<()>"]),
        (Scope::F, &[]), // illegal Desig, but test anyway
    ];

    struct Desigs<'b> {
        owned: Desig,
        borrowed: BorrowedDesig<'b>,
    }
    impl Desigs<'_> {
        fn with((scope, lpaths): &Case, f: impl FnOnce(Desigs<'_>)) {
            let scope = *scope;
            let lpaths = lpaths
                .iter()
                .map(|l| syn::parse_str(l).expect(l))
                .collect_vec();
            let owned = {
                let label = Label {
                    lpaths: lpaths.clone(),
                };
                Desig { scope, label }
            };
            let lpaths_borrowed;
            let borrowed = {
                lpaths_borrowed = lpaths.iter().collect_vec();
                BorrowedDesig {
                    scope,
                    lpaths: &*lpaths_borrowed,
                }
            };
            f(Desigs { owned, borrowed })
        }
    }

    // Test that for each entry in TEST_CASES, when parsed into Paths, etc.,
    // BorrowedDesig is `equivalent` to, and hashes the same as, Desig.

    for case in TEST_CASES {
        Desigs::with(case, |d| {
            assert!(d.borrowed.equivalent(&d.owned));
            assert_eq!(
                TrackingHasher::hash(&d.owned),
                TrackingHasher::hash(&d.borrowed),
            );
        });
    }

    // Compare every TEST_CASES entry with every other entry.
    // See if the owned forms are equal (according to `PartialEq`).
    // Insist that the Borrowed vs owned `equivalent` relation agrees,
    // in both directions.
    // And, if they are equal, insist that the hashes all agree.

    for (case0, case1) in iproduct!(TEST_CASES, TEST_CASES) {
        Desigs::with(case0, |d0| {
            Desigs::with(case1, |d1| {
                let equal = d0.owned == d1.owned;
                assert_eq!(equal, d0.borrowed.equivalent(&d1.owned));
                assert_eq!(equal, d1.borrowed.equivalent(&d0.owned));
                if equal {
                    let hash = TrackingHasher::hash(&d0.owned);
                    assert_eq!(hash, TrackingHasher::hash(&d1.owned));
                    assert_eq!(hash, TrackingHasher::hash(&d0.borrowed));
                    assert_eq!(hash, TrackingHasher::hash(&d1.borrowed));
                }
            });
        });
    }
}

//---------- conditional support for `Xmeta as items` ----------

#[cfg(feature = "meta-as-expr")]
pub type ValueExpr = syn::Expr;
#[cfg(not(feature = "meta-as-expr"))]
pub type ValueExpr = MetaUnsupported;

#[cfg(feature = "meta-as-items")]
pub type ValueItems = Concatenated<syn::Item>;
#[cfg(not(feature = "meta-as-items"))]
pub type ValueItems = MetaUnsupported;

/// newtype to avoid coherence - it doesn't impl `Parse + ToTokens`
#[derive(Debug, Copy, Clone)]
pub struct MetaUnsupported(Void);

#[derive(Debug, Copy, Clone)]
pub struct SubstAsSupported<P: SubstAsSupportStatus>(P::Marker);

/// Implemented for syn types supported in this build, and `MetaUnsupported`
pub trait SubstAsSupportStatus: Sized {
    type Marker;
    type Parsed: Parse + ToTokens;
    fn new(kw: &IdentAny) -> syn::Result<SubstAsSupported<Self>>;
}

impl<P: SubstAsSupportStatus> SubstAsSupported<P> {
    fn infer_type(&self, _parsed: &P::Parsed) {}
}

impl<T: Parse + ToTokens> SubstAsSupportStatus for T {
    type Marker = ();
    type Parsed = T;
    fn new(_kw: &IdentAny) -> syn::Result<SubstAsSupported<Self>> {
        Ok(SubstAsSupported(()))
    }
}

impl SubstAsSupportStatus for MetaUnsupported {
    type Marker = MetaUnsupported;
    type Parsed = TokenStream;
    fn new(kw: &IdentAny) -> syn::Result<SubstAsSupported<Self>> {
        Err(kw.error(format_args!(
            // We're a bit fast and loose here: if kw contained `_`,
            // or there were aliases, this message would be a bit wrong.
            "${{Xmeta as {}}} used but cargo feature meta-as-{} disabled",
            **kw, **kw,
        )))
    }
}

impl ToTokens for MetaUnsupported {
    fn to_tokens(&self, _out: &mut TokenStream) {
        void::unreachable(self.0)
    }
}

//---------- template expansion ----------

impl<O> SubstMeta<O>
where
    O: ExpansionOutput,
    TemplateElement<O>: Expand<O>,
{
    pub fn expand(
        &self,
        ctx: &Context,
        kw_span: Span,
        out: &mut O,
        pmetas: &PreprocessedMetas,
    ) -> syn::Result<()> {
        let SubstMeta {
            desig,
            as_,
            default,
        } = self;
        let mut found = None::<FoundNode>;
        let mut hint = None::<FoundNearbyNode>;
        let span_whole = self.span_whole(kw_span);
        let self_loc = || (span_whole, "expansion");
        let error_loc = || [ctx.error_loc(), self_loc()];

        desig.label.search(
            pmetas,
            &mut |av: FoundNode| {
                if let Some(first) = &found {
                    return Err([(first.path_span, "first occurrence"),
                            (av.path_span, "second occurrence"),
                            self_loc()].error(
 "tried to expand just attribute value, but it was specified multiple times"
                    ));
                }
                found = Some(av);
                Ok(())
            },
            &mut |nearby| {
                hint.get_or_insert(nearby);
                Ok(())
            },
        )?;

        if let (None, Some((def, ..))) = (&found, default) {
            return Ok(def.expand(ctx, out));
        }

        let found = found.ok_or_else(|| {
            if let Some(hint) = hint {
                let hint_msg = match hint.kind {
                    FNNK::Unit =>
 "expected a list with sub-attributes, found a unit",
                    FNNK::Lit =>
 "expected a list with sub-attributes, found a simple value",
                    FNNK::List =>
 "expected a leaf node, found a list with sub-attributes",
                };
                let mut err = hint.path_span.error(hint_msg);
                err.combine(error_loc().error(
 "attribute value expanded, but no suitable value in data structure definition"
                ));
                err
            } else {
                error_loc().error(
 "attribute value expanded, but no value in data structure definition"
                )
            }
        })?;

        found.ptree.update_used(Usage::Value);
        found.expand(span_whole, as_, out)?;

        Ok(())
    }
}

fn metavalue_spans(tspan: Span, vspan: Span) -> [ErrorLoc<'static>; 2] {
    [(vspan, "attribute value"), (tspan, "template")]
}

/// Obtain the `LiStr` from a meta node value (ie, a `Lit`)
///
/// This is the thing we actually use.
/// Non-string-literal values are not allowed.
fn metavalue_litstr<'l>(
    lit: &'l syn::Lit,
    tspan: Span,
    msg: fmt::Arguments<'_>,
) -> syn::Result<&'l syn::LitStr> {
    match lit {
        syn::Lit::Str(s) => Ok(s),
        // having checked derive_builder, it doesn't handle
        // Lit::Verbatim so I guess we don't need to either.
        _ => Err(metavalue_spans(tspan, lit.span()).error(msg)),
    }
}

/// Convert a literal found in a meta item into `T`
///
/// `into_what` is used only for error reporting
pub fn metavalue_lit_as<T>(
    lit: &syn::Lit,
    tspan: Span,
    into_what: &dyn Display,
) -> syn::Result<T>
where
    T: Parse + ToTokens,
{
    let s = metavalue_litstr(
        lit,
        tspan,
        format_args!(
            "expected string literal, for conversion to {}",
            into_what,
        ),
    )?;

    let t: TokenStream = s.parse().map_err(|e| {
        // Empirically, parsing a LitStr in actual proc macro context, with
        // proc_macro2, into tokens, can generate a lexical error with a
        // "fallback" Span.  Then, attempting to render the results,
        // including the eventual compiler_error! invocation, back to
        // a compiler proc_ma cor::TokenStream can panic with
        // "compiler/fallback mismatch".
        //
        // https://github.com/dtolnay/syn/issues/1504
        //
        // Attempt to detect this situation.
        match (|| {
            let _: String = (&e).into_iter().next()?.span().source_text()?;
            Some(())
        })() {
            Some(()) => e,
            None => lit.span().error(e.to_string()),
        }
    })?;

    let thing: T = syn::parse2(t)?;
    Ok(thing)
}

impl<'l> FoundNode<'l> {
    fn expand<O>(
        &self,
        tspan: Span,
        as_: &Option<SubstAs<O>>,
        out: &mut O,
    ) -> syn::Result<()>
    where
        O: ExpansionOutput,
    {
        let spans = |vspan| metavalue_spans(tspan, vspan);

        let lit = match self.kind {
            FNK::Unit => return Err(spans(self.path_span).error(
 "tried to expand attribute which is just a unit, not a literal"
            )),
            FNK::Lit(lit) => lit,
        };

        use SubstAs as SA;

        let default_buf;
        let as_ = match as_ {
            Some(as_) => as_,
            None => {
                default_buf = O::default_subst_meta_as(tspan)?;
                &default_buf
            }
        };

        match as_ {
            as_ @ SA::expr(.., np, supported) => {
                let expr = metavalue_lit_as(lit, tspan, as_)?;
                supported.infer_type(&expr);
                out.append_tokens(np, Grouping::Parens.surround(expr))?;
            }
            as_ @ SA::ident(..) => {
                let ident: IdentAny = metavalue_lit_as(lit, tspan, as_)?;
                out.append_identfrag_toks(&*ident)?;
            }
            SA::items(_, np, supported) => {
                let items = metavalue_lit_as(lit, tspan, &"items")?;
                supported.infer_type(&items);
                out.append_tokens(np, items)?;
            }
            as_ @ SA::path(..) => out.append_syn_type(
                tspan,
                syn::Type::Path(metavalue_lit_as(lit, tspan, as_)?),
                Grouping::Invisible,
            ),
            SA::str(..) => {
                let s = metavalue_litstr(
                    lit,
                    tspan,
                    format_args!("expected string literal, for meta value",),
                )?;
                out.append_syn_litstr(s);
            }
            as_ @ SA::ty(..) => out.append_syn_type(
                tspan,
                metavalue_lit_as(lit, tspan, as_)?,
                Grouping::Invisible,
            ),
            SA::token_stream(_, np) => {
                let tokens: TokenStream =
                    metavalue_lit_as(lit, tspan, &"tokens")?;
                out.append_tokens(np, tokens)?;
            }
        }
        Ok(())
    }
}

//==================== implementations - usage checking ====================

impl Parse for CheckUsed<UsedGroup> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let la = input.lookahead1();
        Ok(if la.peek(Token![*]) {
            let _star: Token![*] = input.parse()?;
            mCU::Unchecked
        } else if la.peek(token::Bracket) {
            let group: proc_macro2::Group = input.parse()?;
            let content = group.stream();
            mCU::Check(UsedGroup { content })
        } else {
            return Err(la.error());
        })
    }
}

impl Recognised {
    /// Ensures that `self[k] >= v`
    pub fn update(&mut self, k: Desig, v: Usage) {
        let ent = self.map.entry(k).or_insert(v);
        *ent = cmp::max(*ent, v);
    }
}

impl ToTokens for Recognised {
    fn to_tokens(&self, out: &mut TokenStream) {
        for (desig, allow) in &self.map {
            match allow {
                U::BoolOnly => {
                    out.extend(quote! { ? });
                }
                U::Value => {}
            }
            desig.to_tokens(Span::call_site(), out);
        }
    }
}

impl PreprocessedTree {
    pub fn update_used(&self, ra: Usage) {
        self.used.set(cmp::max(self.used.get(), Some(ra)));
    }
}

//---------- decoding used metas ----------

impl PreprocessedValueList {
    fn decode_update_used(&self, input: ParseStream) -> syn::Result<()> {
        use PreprocessedValue as PV;

        for ptree in &self.content {
            if input.is_empty() {
                return Ok(());
            }
            if !input.peek(Token![,]) {
                let path = input.call(syn::Path::parse_mod_style)?;
                if path != ptree.path {
                    return Err([
                        (path.span(), "found"),
                        (ptree.path.span(), "expected"),
                    ].error(
 "mismatch (desynchronised) incorporating previous expansions' used metas"
                    ));
                }
                let used = if input.peek(Token![=]) {
                    let _: Token![=] = input.parse()?;
                    Some(U::Value)
                } else if input.peek(Token![?]) {
                    let _: Token![?] = input.parse()?;
                    Some(U::BoolOnly)
                } else {
                    None
                };
                if let Some(used) = used {
                    ptree.update_used(used);
                }
                if input.peek(token::Paren) {
                    let inner;
                    let paren = parenthesized!(inner in input);
                    let sub_list = match &ptree.value {
                        PV::Unit | PV::Value { .. } => return Err([
                            (paren.span.open(), "found"),
                            (ptree.path.span(), "defined"),
                        ].error(
 "mismatch (tree vs terminal) incorporating previous expansions' used metas"
                        )),
                        PV::List(l) => l,
                    };
                    sub_list.decode_update_used(&inner)?;
                }
            }
            if input.is_empty() {
                return Ok(());
            }
            let _: Token![,] = input.parse()?;
        }
        Ok(())
    }
}

impl<'c> Context<'c> {
    pub fn decode_update_metas_used(
        &self,
        input: /* group content */ ParseStream,
    ) -> syn::Result<()> {
        #[derive(Default)]
        struct Intended {
            variant: Option<syn::Ident>,
            field: Option<Either<syn::Ident, u32>>,
            attr_i: usize,
        }
        let mut intended = Intended::default();

        let mut visit =
            |pmetas: &PreprocessedMetas,
             current_variant: Option<&syn::Ident>,
             current_field: Option<Either<&syn::Ident, &u32>>| {
                loop {
                    let la = input.lookahead1();
                    if input.is_empty() {
                        // keep visiting until we exit all the loops
                        return Ok(());
                    } else if la.peek(Token![::]) {
                        let _: Token![::] = input.parse()?;
                        intended = Intended {
                            variant: Some(input.parse()?),
                            field: None,
                            attr_i: 0,
                        };
                    } else if la.peek(Token![.]) {
                        let _: Token![.] = input.parse()?;
                        intended.field = Some(match input.parse()? {
                            syn::Member::Named(n) => Either::Left(n),
                            syn::Member::Unnamed(i) => Either::Right(i.index),
                        });
                        intended.attr_i = 0;
                    } else if {
                        let intended_field_refish = intended
                            .field
                            .as_ref()
                            .map(|some: &Either<_, _>| some.as_ref());

                        !(current_variant == intended.variant.as_ref()
                            && current_field == intended_field_refish)
                    } {
                        // visit subsequent things, hopefully one will match
                        return Ok(());
                    } else if la.peek(token::Paren) {
                        // we're in the right place and have found a #[deftly()]
                        let i = intended.attr_i;
                        intended.attr_i += 1;
                        let m = pmetas.get(i).ok_or_else(|| {
                            input.error("more used metas, out of range!")
                        })?;
                        let r;
                        let _ = parenthesized!(r in input);
                        m.decode_update_used(&r)?;
                    } else {
                        return Err(la.error());
                    }
                }
            };

        visit(&self.pmetas, None, None)?;

        WithinVariant::for_each(self, |ctx, wv| {
            let current_variant = wv.variant.map(|wv| &wv.ident);

            if !wv.is_struct_toplevel_as_variant() {
                visit(&wv.pmetas, current_variant, None)?;
            }

            WithinField::for_each(ctx, |_ctx, wf| {
                let current_field = if let Some(ref ident) = wf.field.ident {
                    Either::Left(ident)
                } else {
                    Either::Right(&wf.index)
                };

                visit(&wf.pfield.pmetas, current_variant, Some(current_field))
            })
        })

        // if we didn't consume all of the input, due to mismatches/
        // misordering, then syn will give an error for us
    }
}

//---------- encoding used metas ---------

impl PreprocessedTree {
    /// Returns `(....)`
    fn encode_useds(
        list: &PreprocessedValueList,
    ) -> Option<proc_macro2::Group> {
        let preamble = syn::parse::Nothing;
        let sep = Token![,](Span::call_site());
        let mut ts = TokenStream::new();
        let mut ot = TokenOutputTrimmer::new(&mut ts, &preamble, &sep);
        for t in &list.content {
            t.encode_used(&mut ot);
            ot.push_sep();
        }
        if ts.is_empty() {
            None
        } else {
            Some(proc_macro2::Group::new(Delimiter::Parenthesis, ts))
        }
    }

    /// Writes `path?=(...)` (or, rather, the parts of it that are needed)
    fn encode_used(&self, out: &mut TokenOutputTrimmer) {
        use PreprocessedValue as PV;

        struct OutputTrimmerWrapper<'or, 'o, 't, 'p> {
            // None if we have written the path already
            path: Option<&'p syn::Path>,
            out: &'or mut TokenOutputTrimmer<'t, 'o>,
        }

        let mut out = OutputTrimmerWrapper {
            path: Some(&self.path),
            out,
        };

        impl OutputTrimmerWrapper<'_, '_, '_, '_> {
            fn push_reified(&mut self, t: &dyn ToTokens) {
                if let Some(path) = self.path.take() {
                    self.out.push_reified(path);
                }
                self.out.push_reified(t);
            }
        }

        let tspan = Span::call_site();

        if let Some(used) = self.used.get() {
            match used {
                U::BoolOnly => out.push_reified(&Token![?](tspan)),
                U::Value => out.push_reified(&Token![=](tspan)),
            }
        }

        match &self.value {
            PV::Unit | PV::Value { .. } => {}
            PV::List(l) => {
                if let Some(group) = PreprocessedTree::encode_useds(l) {
                    out.push_reified(&group);
                }
            }
        }
    }
}

impl<'c> Context<'c> {
    /// Returns `[::Variant .field () ...]`
    pub fn encode_metas_used(&self) -> proc_macro2::Group {
        let parenthesize =
            |ts| proc_macro2::Group::new(Delimiter::Parenthesis, ts);
        let an_empty = parenthesize(TokenStream::new());

        let mut ts = TokenStream::new();

        struct Preamble<'p> {
            variant: Option<&'p syn::Variant>,
            field: Option<&'p WithinField<'p>>,
        }
        impl ToTokens for Preamble<'_> {
            fn to_tokens(&self, out: &mut TokenStream) {
                let span = Span::call_site();
                if let Some(v) = self.variant {
                    Token![::](span).to_tokens(out);
                    v.ident.to_tokens(out);
                }
                if let Some(f) = self.field {
                    Token![.](span).to_tokens(out);
                    f.fname(span).to_tokens(out);
                }
            }
        }

        let mut last_variant: *const syn::Variant = ptr::null();
        let mut last_field: *const syn::Field = ptr::null();

        fn ptr_of_ref<'i, InDi>(r: Option<&'i InDi>) -> *const InDi {
            r.map(|r| r as _).unwrap_or_else(ptr::null)
        }

        let mut encode = |pmetas: &PreprocessedMetas,
                          wv: Option<&WithinVariant>,
                          wf: Option<&WithinField>| {
            let now_variant: *const syn::Variant =
                ptr_of_ref(wv.map(|wv| wv.variant).flatten());
            let now_field: *const syn::Field =
                ptr_of_ref(wf.map(|wf| wf.field));

            let preamble = Preamble {
                variant: (!ptr::eq(last_variant, now_variant)).then(|| {
                    last_field = ptr::null();
                    let v = wv.expect("had WithinVariant, now not");
                    v.variant.expect("variant was syn::Variant, now not")
                }),
                field: (!ptr::eq(last_field, now_field)).then(|| {
                    wf.expect("had WithinField (Field), now not") //
                }),
            };
            let mut ot =
                TokenOutputTrimmer::new(&mut ts, &preamble, &an_empty);
            for m in pmetas {
                if let Some(group) = PreprocessedTree::encode_useds(m) {
                    ot.push_reified(group);
                } else {
                    ot.push_sep();
                }
            }
            if ot.did_preamble().is_some() {
                last_variant = now_variant;
                last_field = now_field;
            }

            Ok::<_, Void>(())
        };

        encode(&self.pmetas, None, None).void_unwrap();
        WithinVariant::for_each(self, |ctx, wv| {
            if !wv.is_struct_toplevel_as_variant() {
                encode(&wv.pmetas, Some(wv), None)?;
            }
            WithinField::for_each(ctx, |_ctx, wf| {
                encode(&wf.pfield.pmetas, Some(wv), Some(wf))
            })
        })
        .void_unwrap();

        proc_macro2::Group::new(Delimiter::Bracket, ts)
    }
}

//---------- checking used metas ----------

struct UsedChecker<'c, 'e> {
    current: Vec<&'c syn::Path>,
    reported: &'e mut HashSet<Label>,
    recog: &'c Recognised,
    supplied_scope: SuppliedScope,
    errors: &'e mut ErrorAccumulator,
}

impl PreprocessedTree {
    fn check_used<'c>(&'c self, checker: &mut UsedChecker<'c, '_>) {
        checker.current.push(&self.path);

        let mut err = |err| {
            let lpaths = checker.current.iter().copied().cloned().collect();
            if checker.reported.insert(Label { lpaths }) {
                checker.errors.push(err);
            }
        };

        let ur_err = |supplied_want_allow, used_allow| {
            unrecognised_error(
                checker.recog,
                checker.supplied_scope,
                supplied_want_allow,
                self.path.span(),
                used_allow,
                &checker.current,
            )
            .void_unwrap_err()
        };

        match (&self.value, self.used.get()) {
            (PreprocessedValue::Unit, Some(_)) => {}
            (PreprocessedValue::Value { .. }, Some(U::Value)) => {}
            (PreprocessedValue::List(l), _) => {
                if l.content.is_empty() {
                    err(self.path.error(
 "empty nested list in #[deftly], is not useable by any template"
                    ));
                }
                for subtree in &l.content {
                    subtree.check_used(checker);
                }
            }
            (PreprocessedValue::Value { .. }, u @ Some(U::BoolOnly))
            | (PreprocessedValue::Value { .. }, u @ None) => {
                err(ur_err(U::Value, u));
            }
            (PreprocessedValue::Unit, u @ None) => {
                err(ur_err(U::BoolOnly, u));
            }
        }

        checker
            .current
            .pop()
            .expect("pushed earlier, but can't pop?");
    }
}

impl<'c> Context<'c> {
    pub(crate) fn check_metas_used(
        &self,
        errors: &mut ErrorAccumulator,
        recog: &Recognised,
    ) {
        use SuppliedScope as SS;

        let mut reported = HashSet::new();

        let mut chk_pmetas = |supplied_scope, pmetas: &PreprocessedMetas| {
            let mut checker = UsedChecker {
                reported: &mut reported,
                current: vec![],
                recog,
                errors,
                supplied_scope,
            };

            for a in pmetas {
                for l in &a.content {
                    l.check_used(&mut checker);
                }
            }

            Ok::<_, Void>(())
        };

        chk_pmetas(
            match &self.top.data {
                syn::Data::Struct(_) | syn::Data::Union(_) => SS::Struct,
                syn::Data::Enum(_) => SS::Enum,
            },
            &self.pmetas,
        )
        .void_unwrap();

        WithinVariant::for_each(self, |ctx, wv| {
            // If variant is None, this is the imaginary variant for
            // the toplevel, and it has a copy of the ref to the metas,
            // in which case we don't want to process it again.
            if !wv.is_struct_toplevel_as_variant() {
                chk_pmetas(SS::Variant, &wv.pmetas)?;
            }
            WithinField::for_each(ctx, |_ctx, wf| {
                chk_pmetas(SS::Field, &wf.pfield.pmetas)
            })
        })
        .void_unwrap();
    }
}

fn unrecognised_error(
    recog: &Recognised,
    supplied_scope: SuppliedScope,
    supplied_usage: Usage,
    supplied_span: Span,
    used_usage: Option<Usage>,
    lpaths: &[&syn::Path],
) -> Result<Void, syn::Error> /* return type allows (ab)use of ? */ {
    // This could have been a method on UsedChecker, but that runs into
    // borrowck trouble.

    let try_case = |e: Option<_>| e.map(Err).unwrap_or(Ok(()));
    let some_err = |m: &dyn Display| Some(supplied_span.error(m));

    // Maybe this would have been recognised in other circumstances.
    // If so, report that.
    try_case((|| {
        let recog_allow = supplied_scope
            .recog_search()
            .map(|scope| {
                recog.map.get(&BorrowedDesig { scope, lpaths }).copied()
            })
            .max()
            .unwrap_or_default()?;

        if recog_allow < supplied_usage {
            // We've supplied more than would ever be recognised
            return None;
        }

        // We'd recognise this, but only conditionally, and not now

        match used_usage {
            None => some_err(&format_args!(
 "meta attribute provided, and (conditionally) recognised; but not used in these particular circumstances"
            )),
            Some(U::BoolOnly) => some_err(&format_args!(
 "meta attribute provided with value, and (conditionally) recognised with value; but in these particular circumstances only used as a boolean"
            )),
            Some(U::Value) => unreachable!(),
        }
    })())?;

    // Now we know that even the static scan doesn't
    // recognise this item.

    // Maybe it's just that a value was supplied by mistake
    try_case((|| {
        //
        match (supplied_usage, used_usage) {
            (U::Value, Some(U::BoolOnly)) => some_err(&format_args!(
                "meta attribute value provided, but is used only as a boolean",
            )),
            (_, None) => None,
            (U::BoolOnly, Some(_)) => unreachable!(),
            (U::Value, Some(U::Value)) => unreachable!(),
        }
    })())?;

    // Look to see if it would have been recognised in
    // another scope.  That would mean the attr is
    // merely misplaced, rather than totally wrong.
    try_case((|| {
        let y_scopes = Scope::iter()
            .filter(|&scope| {
                recog.map.contains_key(&BorrowedDesig { scope, lpaths })
            })
            .collect_vec();

        if y_scopes.is_empty() {
            return None;
        }
        let y_ss = SuppliedScope::iter()
            .filter(|ss| ss.recog_search().any(|s| y_scopes.contains(&s)))
            .map(|ss| ss.to_string())
            .join("/");
        let y_scopes = y_scopes.iter().map(|s| s.as_ref()).join("/");
        some_err(&format_args!(
 "meta attribute provided for {}, but recognised by template only for {} ({})",
            supplied_scope,
            y_ss,
            y_scopes,
        ))
    })())?;

    // Look to see if user supplied `#[deftly(foo(bar))]`
    // when the template wanted `#[deftly(foo = "bar")]`.
    try_case((|| {
        let (upper, recog) =
            supplied_scope.recog_search().find_map(|scope| {
                let upper = BorrowedDesig {
                    scope,
                    lpaths: lpaths.split_last()?.1,
                };
                let recog = recog.map.get(&upper)?;
                Some((upper, recog))
            })?;
        if *recog != U::Value {
            return None;
        }
        some_err(&format_args!(
 "nested meta provided and not recognised; but, template would recognise {}",
            DisplayAsIfSpecified {
                lpaths: upper.lpaths,
                inside_after: " = ..",
            },
        ))
    })())?;

    // Look to see if the specified attribute is a prefix of
    // a recognised one
    try_case((|| {
        recog.map.keys().find_map(|desig| {
            // deliberately ignore scope
            if !itertools::equal(
                lpaths.iter().copied(),
                desig.label.lpaths.iter().take(lpaths.len()),
            ) {
                return None;
            }
            Some(())
        })?;
        some_err(&format_args!(
 "meta attribute provided, but not recognised; template only uses it as a container"
        ))
    })())?;

    Err(supplied_span
        .error("meta attribute provided, but not recognised by template"))
}

//---------- impls of FindRecogMetas ----------

impl<O: SubstParseContext> FindRecogMetas for Template<O> {
    fn find_recog_metas(&self, acc: &mut Recognised) {
        for e in &self.elements {
            e.find_recog_metas(acc)
        }
    }
}

impl<O: SubstParseContext> FindRecogMetas for TemplateElement<O> {
    fn find_recog_metas(&self, acc: &mut Recognised) {
        match self {
            TE::Ident(_) | TE::LitStr(_) | TE::Literal(..) | TE::Punct(..) => {
            }
            TE::Group { template, .. } => template.find_recog_metas(acc),
            TE::Subst(n) => n.find_recog_metas(acc),
            TE::Repeat(n) => n.find_recog_metas(acc),
        }
    }
}

impl<O: SubstParseContext> FindRecogMetas for RepeatedTemplate<O> {
    fn find_recog_metas(&self, acc: &mut Recognised) {
        self.template.find_recog_metas(acc);
        self.whens.find_recog_metas(acc);
    }
}

impl<O: SubstParseContext> FindRecogMetas for Subst<O> {
    fn find_recog_metas(&self, acc: &mut Recognised) {
        self.sd.find_recog_metas(acc);
    }
}

macro_rules! impL_find_recog_metas_via_iter { {
    ( $($gens:tt)* ), $i:ty
} => {
    impl<T: FindRecogMetas, $($gens)*> FindRecogMetas for $i {
        fn find_recog_metas(&self, acc: &mut Recognised) {
            #[allow(for_loops_over_fallibles)]
            for item in self {
                item.find_recog_metas(acc)
            }
        }
    }
} }
impL_find_recog_metas_via_iter!((), [T]);
impL_find_recog_metas_via_iter!((), Option<T>);
impL_find_recog_metas_via_iter!((U), Punctuated<T, U>);

macro_rules! impL_find_recog_metas_via_deref { {
    ( $($gens:tt)* ), $i:ty
} => {
    impl<$($gens)*> FindRecogMetas for $i {
        fn find_recog_metas(&self, acc: &mut Recognised) {
            (**self).find_recog_metas(acc)
        }
    }
} }
impL_find_recog_metas_via_deref!((O: SubstParseContext), Argument<O>);
impL_find_recog_metas_via_deref!((T: FindRecogMetas), Box<T>);

impl<O: SubstParseContext> FindRecogMetas for SubstDetails<O> {
    fn find_recog_metas(&self, acc: &mut Recognised) {
        use SD::*;

        match self {
            Xmeta(v) => v.find_recog_metas(acc),
            vpat(v, _, _) => v.find_recog_metas(acc),
            vtype(v, _, _) => v.find_recog_metas(acc),
            tdefvariants(v, _, _) => v.find_recog_metas(acc),
            fdefine(v, _, _) => v.find_recog_metas(acc),
            vdefbody(a, b, _, _) => {
                a.find_recog_metas(acc);
                b.find_recog_metas(acc);
            }
            paste(v, _) => v.find_recog_metas(acc),
            ChangeCase(v, _, _) => v.find_recog_metas(acc),

            when(v, _) => v.find_recog_metas(acc),
            define(v, _) => v.find_recog_metas(acc),
            defcond(v, _) => v.find_recog_metas(acc),

            not(v, _) => v.find_recog_metas(acc),
            any(v, _) | all(v, _) => v.find_recog_metas(acc),
            is_empty(_, v) => v.find_recog_metas(acc),
            approx_equal(_, v) => v.find_recog_metas(acc),

            For(v, _) => v.find_recog_metas(acc),
            If(v, _) | select1(v, _) => v.find_recog_metas(acc),
            ignore(v, _) => v.find_recog_metas(acc),
            dbg(v) => v.content_parsed.find_recog_metas(acc),

            tname(_)
            | ttype(_)
            | tdeftype(_)
            | vname(_)
            | fname(_)
            | ftype(_)
            | fpatname(_)
            | tdefkwd(_)
            | Vis(_, _)
            | tattrs(_, _, _)
            | vattrs(_, _, _)
            | fattrs(_, _, _)
            | tgens(_)
            | tdefgens(_, _)
            | tgnames(_, _)
            | twheres(_, _)
            | UserDefined(_)
            | False(_)
            | True(_)
            | is_struct(_)
            | is_enum(_)
            | is_union(_)
            | v_is_unit(_)
            | v_is_tuple(_)
            | v_is_named(_)
            | error(..)
            | dbg_all_keywords(_)
            | Crate(_, _) => {}
        }
    }
}

impl<O: SubstParseContext> FindRecogMetas for SubstMeta<O> {
    fn find_recog_metas(&self, acc: &mut Recognised) {
        let recog = O::meta_recog_usage();
        acc.update(self.desig.clone(), recog);
    }
}

impl FindRecogMetas for SubstVType {
    fn find_recog_metas(&self, acc: &mut Recognised) {
        let Self { self_, vname } = self;
        self_.find_recog_metas(acc);
        vname.find_recog_metas(acc);
    }
}

impl FindRecogMetas for SubstVPat {
    fn find_recog_metas(&self, acc: &mut Recognised) {
        let Self { vtype, fprefix } = self;
        vtype.find_recog_metas(acc);
        fprefix.find_recog_metas(acc);
    }
}

impl<B: FindRecogMetas> FindRecogMetas for Definition<B> {
    fn find_recog_metas(&self, acc: &mut Recognised) {
        let Self {
            name: _,
            body_span: _,
            body,
        } = self;
        body.find_recog_metas(acc);
    }
}

impl FindRecogMetas for DefinitionBody {
    fn find_recog_metas(&self, acc: &mut Recognised) {
        match self {
            DefinitionBody::Normal(v) => v.find_recog_metas(acc),
            DefinitionBody::Paste(v) => v.find_recog_metas(acc),
        }
    }
}

impl<O: SubstParseContext> FindRecogMetas for SubstIf<O> {
    fn find_recog_metas(&self, acc: &mut Recognised) {
        let Self {
            tests,
            otherwise,
            kw_span: _,
        } = self;
        for (if_, then) in tests {
            if_.find_recog_metas(acc);
            then.find_recog_metas(acc);
        }
        otherwise.find_recog_metas(acc);
    }
}
