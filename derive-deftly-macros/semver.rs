//! Macro impl for `template_export_semver_check!`

use super::prelude::*;

#[derive(Debug, Clone)]
struct SemverCheckInput {
    span: Span,
    major: u32,
    minor: u32,
    patch: u32,
}

impl Display for SemverCheckInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.x", self.major, self.minor)
    }
}

impl Parse for SemverCheckInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lit: syn::LitStr = input.parse()?;
        let span = lit.span();
        let s = lit.value();
        let mut nums = s.split('.');

        let mut next_num = || {
            let n = match nums.next() {
                Some(y) => y,
                None => return Ok(None),
            };
            let n = n.parse().map_err(|e| {
                span.error(format_args!("bad number: {}", e)) //
            })?;
            Ok::<_, syn::Error>(Some(n))
        };
        let mut next_num_must = || {
            next_num()?.ok_or_else(|| {
                span.error("too few components in version number")
            })
        };
        let major = next_num_must()?;
        let minor = next_num_must()?;
        let patch = next_num()?.unwrap_or(0);
        if nums.next().is_some() {
            return Err(span.error("too many components in version number"));
        }
        Ok(SemverCheckInput {
            major,
            minor,
            patch,
            span,
        })
    }
}

impl Spanned for SemverCheckInput {
    fn span(&self) -> Span {
        self.span
    }
}

/// This is `template_export_semver_check!`
pub fn template_export_semver_check_func_macro(
    input: TokenStream,
) -> Result<TokenStream, syn::Error> {
    let version: SemverCheckInput = syn::parse2(input)?;

    let mk_err = |e: &dyn Display| Err(version.error(e));
    let too_new = || {
        mk_err(
            &"declared version is greater than this version of derive-deftly!",
        )
    };
    let too_old = || {
        mk_err(&format_args!(
            r#"derive-deftly has been updated!
Please check the changelog for breaking changes since {}
Search for section titles containing "template export semver"."#,
            version,
        ))
    };

    match (version.major, version.minor, version.patch) {
        (0, 0..=11, _) => return too_old(),
        // We tolerate "too new" within the same minor version,
        // since otherwise updating the minor version is too tedious.
        // There are tests to catch failure to update this.
        (0, 12, 0) => return too_old(),
        (0, 12, 1..) => {}
        (0, 13.., _) => {}
        (1.., _, _) => return too_new(),
    }

    Ok(TokenStream::new())
}
