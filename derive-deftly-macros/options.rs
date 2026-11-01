//! "Options", keyword modifiers for an expansion
//!
//! These "combine": multiple specifications of the same option
//! are allowed, so long as they are compatible.

use super::prelude::*;
use adviseable::*;

use OptionDetails as OD;

//---------- types, ordered from general to specific ----------

/// Where are we finding these options?
#[derive(Debug, Copy, Clone)]
pub enum OpContext {
    /// Just before a precanned template
    ///
    /// `define_derivae_deftly!{ Template OPTIONS = ...`
    TemplateDefinition,
    /// Just before an adhoc template
    ///
    /// `derive_deftly_adhoc!{ Driver OPTIONS: ...`
    TemplateAdhoc,
    /// In the driver's application
    ///
    /// `#[derive_deftly(Template[OPTIONS])]`
    DriverApplicationCapture,
    /// Driver's application options as they appear in the ultimate expansion
    ///
    /// With the versioning, so `1 1 AOPTIONS`.
    DriverApplicationPassed,
}

/// All the template options, as a tokenstream, but sanity-checked
///
/// These have been syntax checked, but not semantically checked.
/// The purpose of the syntax check is to get syntax errors early,
/// when a template is defined - rather than when it's applied.
///
/// This also helps with cross-crate compatibility.
#[derive(Default, Debug, Clone)]
pub struct UnprocessedOptions {
    ts: TokenStream,
    pub beta_enabled: Option<beta::Enabled>,
}

/// Template options, semantically resolved
#[derive(Default, Debug, Clone)]
pub struct DdOptions {
    pub dbg: bool,
    pub driver_kind: Option<DdOptVal<ExpectedDriverKind>>,
    pub expect_target: Option<DdOptVal<check::Target>>,
    pub beta_enabled: Option<beta::Enabled>,
}

/// A single template option
#[derive(Debug)]
struct DdOption {
    pub kw_span: Span,
    pub od: OptionDetails,
}

/// Enum for the details of a template option
#[derive(Debug, Clone)]
#[allow(non_camel_case_types)] // clearer to use the exact ident
enum OptionDetails {
    dbg,
    For(DdOptVal<ExpectedDriverKind>),
    expect(DdOptVal<check::Target>),
    beta_deftly(beta::Enabled),
}

/// Value for an option
///
/// If `V` is `FromStr` and `DdOptValDescribable`,
/// this is `Parse`, taking a single keyword.
#[derive(Debug, Clone, Copy)]
pub struct DdOptVal<V> {
    pub value: V,
    pub span: Span,
}

/// Things that go into a `DdOptVal`
pub trait DdOptValDescribable {
    const DESCRIPTION: &'static str;
}

/// The (single) expected driver kind
//
// At some future point, we may want `for ...` keywords that
// specify a range of possible drivers.  Then we'll need both
// this enum, and a *set* of it, and calculate the intersection
// in update_from_option.
#[derive(Debug, Clone, Copy, Eq, PartialEq, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
pub enum ExpectedDriverKind {
    Struct,
    Enum,
    Union,
}

#[derive(Debug, Copy, Clone)]
pub struct OpCompatVersions {
    /// Increased when we make a wholly-incompatible change
    ///
    /// Bumping this will cause rejection of AOPTIONS by old d-d engines.
    /// Hopefully a newer d-d will able to cope with both.
    major: OpCompatVersionNumber,

    /// Increased when we make a more subtle change
    ///
    /// Current d-d versions will ignore this.
    /// Bumping it can be used to psas information
    /// from a newer capturing d-d to a newer template/driver d-d.
    minor: OpCompatVersionNumber,

    /// Span for error reporting
    span: Span,
}
type OpCompatVersionNumber = u32;
impl OpCompatVersions {
    pub fn ours() -> Self {
        OpCompatVersions {
            major: 1,
            minor: 0,
            span: Span::call_site(),
        }
    }
}

impl DdOptValDescribable for ExpectedDriverKind {
    const DESCRIPTION: &'static str = "expected driver kind (in `for` option)";
}

//---------- parsing ----------

impl OpContext {
    fn allowed(self, option: &DdOption) -> syn::Result<()> {
        use OpContext as OC;
        match &option.od {
            OD::dbg => return Ok(()),
            OD::expect(v) => return check::check_expect_opcontext(v, self),
            OD::For(..) => {}
            OD::beta_deftly(..) => {}
        }
        match self {
            OC::TemplateDefinition => Ok(()),
            OC::TemplateAdhoc => Ok(()),
            OC::DriverApplicationCapture | OC::DriverApplicationPassed => {
                Err(option.kw_span.error(
                    "this derive-deftly option is only supported in templates",
                ))
            }
        }
    }

    fn parse_versions(
        self,
        input: ParseStream,
    ) -> syn::Result<OpCompatVersions> {
        use OpContext as OC;
        let ours = OpCompatVersions::ours();
        let got = match self {
            OC::TemplateDefinition
            | OC::TemplateAdhoc
            | OC::DriverApplicationCapture => ours,
            OC::DriverApplicationPassed => input.parse()?,
        };
        if got.major != ours.major {
            return Err(got.error(format_args!(
 "Incompatible major version for AOPTIONS (driver {}, template/engine {})",
                got.major,
                ours.major,
            )));
        }
        Ok(got)
    }
}

impl ToTokens for OpCompatVersions {
    fn to_tokens(&self, out: &mut TokenStream) {
        let OpCompatVersions { major, minor, span } = OpCompatVersions::ours();
        out.extend(quote_spanned! {span=> #major #minor });
    }
}

impl Parse for OpCompatVersions {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let number = move || {
            let lit: syn::LitInt = input.parse()?;
            Ok::<_, syn::Error>((lit.span(), lit.base10_parse()?))
        };
        let (span, major) = number()?;
        let (_, minor) = number()?;
        Ok(OpCompatVersions { major, minor, span })
    }
}

fn continue_options(input: ParseStream) -> Option<Lookahead1> {
    if input.is_empty() {
        return None;
    }
    let la = input.lookahead1();
    if la.peek(Token![:]) || la.peek(Token![=]) {
        return None;
    }
    Some(la)
}

impl UnprocessedOptions {
    pub fn parse(
        input: ParseStream,
        opcontext: OpContext,
    ) -> syn::Result<Self> {
        let mut beta_enabled = None;

        // Scan ahead for a syntax check (and beta enablement)
        DdOption::parse_several(&input.fork(), opcontext, |opt| {
            match &opt.od {
                OD::beta_deftly(be) => beta_enabled = Some(*be),
                _ => {}
            };
            Ok(())
        })?;

        // Collect everything until the : or =
        let mut out = TokenStream::new();
        while continue_options(input).is_some() {
            let tt: TokenTree = input.parse()?;
            out.extend([tt]);
        }
        Ok(UnprocessedOptions {
            ts: out,
            beta_enabled,
        })
    }
}

impl DdOptions {
    pub fn parse_update(
        &mut self,
        input: ParseStream,
        opcontext: OpContext,
    ) -> syn::Result<()> {
        DdOption::parse_several(input, opcontext, |option| {
            self.update_from_option(option)
        })
    }
}

impl DdOption {
    /// Parses zero or more options, in `opcontext`
    ///
    /// With `OpContext::DriverApplicationPassed`,
    /// expects to find the version information too.
    fn parse_several(
        input: ParseStream,
        opcontext: OpContext,
        mut each: impl FnMut(DdOption) -> syn::Result<()>,
    ) -> syn::Result<()> {
        let _versions = opcontext
            .parse_versions(input)
            .map_err(advise_incompatibility)?;

        while let Some(la) = continue_options(input) {
            if !la.peek(Ident::peek_any) {
                return Err(la.error());
            }
            let option = input.parse()?;
            opcontext.allowed(&option)?;
            each(option)?;

            let la = if let Some(la) = continue_options(input) {
                la
            } else {
                break;
            };
            if !la.peek(Token![,]) {
                return Err(la.error());
            }
            let _: Token![,] = input.parse()?;
        }
        Ok(())
    }
}

impl Parse for DdOption {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let kw: IdentAny = input.parse()?;

        let from_od = |od| {
            Ok(DdOption {
                kw_span: kw.span(),
                od,
            })
        };

        // See keyword_general! in utils.rs
        macro_rules! keyword { { $($args:tt)* } => {
            keyword_general! { kw from_od OD; $($args)* }
        } }

        keyword! { dbg }
        keyword! { "for": For(input.parse()?) }
        keyword! { expect(input.parse()?) }
        keyword! { beta_deftly(beta::Enabled::new_for_dd_option(kw.span())?) }

        Err(kw.error("unknown derive-deftly option"))
    }
}

impl<V: FromStr + DdOptValDescribable> Parse for DdOptVal<V> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let kw: IdentAny = input.parse()?;
        let value = kw.to_string().parse().map_err(|_| {
            kw.error(format_args!("unknown value for {}", V::DESCRIPTION))
        })?;
        let span = kw.span();
        Ok(DdOptVal { value, span })
    }
}

//---------- processing ----------

impl ToTokens for UnprocessedOptions {
    fn to_tokens(&self, out: &mut TokenStream) {
        out.extend(self.ts.clone());
    }
}

impl UnprocessedOptions {
    #[allow(dead_code)] // Currently unused, retain it in case we need it
    pub fn is_empty(&self) -> bool {
        self.ts.is_empty()
    }
}

impl DdOptions {
    /// Update `self` according to the option specified in `option`
    ///
    /// On error (eg, contradictory options), fails.
    fn update_from_option(&mut self, option: DdOption) -> syn::Result<()> {
        fn store<V>(
            already: &mut Option<DdOptVal<V>>,
            new: DdOptVal<V>,
        ) -> syn::Result<()>
        where
            V: PartialEq + DdOptValDescribable,
        {
            match already {
                Some(already) if already.value == new.value => Ok(()),
                Some(already) => {
                    Err([(already.span, "first"), (new.span, "second")].error(
                        format_args!(
                            "contradictory values for {}",
                            V::DESCRIPTION,
                        ),
                    ))
                }
                None => {
                    *already = Some(new);
                    Ok(())
                }
            }
        }

        Ok(match option.od {
            OD::dbg => self.dbg = true,
            OD::expect(spec) => store(&mut self.expect_target, spec)?,
            OD::For(spec) => store(&mut self.driver_kind, spec)?,
            OD::beta_deftly(be) => self.beta_enabled = Some(be),
        })
    }
}
