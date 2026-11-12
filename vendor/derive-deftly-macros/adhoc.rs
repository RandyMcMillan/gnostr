//! Macro impl for adhoc template application `derive_deftly_adhoc!`

use super::prelude::*;

#[derive(Debug, Clone)]
struct TemplateInvocation {
    driver: syn::Path,
    options: UnprocessedOptions,
    colon: Token![:],
    template: TokenStream,
}

impl Parse for TemplateInvocation {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let driver = input.parse()?;
        let options =
            UnprocessedOptions::parse(&input, OpContext::TemplateAdhoc)?;
        let colon = input.parse()?;
        let template = input.parse()?;
        Ok(TemplateInvocation {
            driver,
            options,
            colon,
            template,
        })
    }
}

/// This is `derive_deftly_adhoc!`
///
/// It parses
/// ```rust,ignore
///    StructName:
///    SOME_TOKENS
/// ```
/// and expand it to an invocation of
/// ```rust,ignore
///    derive_deftly_driver_StructName
/// ```
/// as per NOTES.txt.
pub fn derive_deftly_adhoc(
    input: TokenStream,
) -> Result<TokenStream, syn::Error> {
    let TemplateInvocation {
        driver,
        options,
        colon,
        template,
    } = syn::parse2(input)?;

    dprint_block!(
        [&driver.to_token_stream(), &template],
        "derive_deftly_adhoc! input",
    );

    let driver_mac_name = {
        let mut name = driver;
        let last = name.segments.last_mut().ok_or_else(|| {
            colon.error(
                "expected non-empty path for driver struct name, found colon",
            )
        })?;
        last.ident = format_ident!("derive_deftly_driver_{}", &last.ident);
        name
    };

    let output = quote! {
        #driver_mac_name !{
            { #template }
            { ($) }
            ( crate; [#options] ; )
            [] []
        }
    };

    dprint_block!(&output, "derive_deftly_adhoc! output");

    Ok(output)
}
