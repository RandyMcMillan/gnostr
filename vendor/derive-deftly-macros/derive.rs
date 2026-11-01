//! Macro impl for capturing the driver `#[derive(Deftly)]`

use super::prelude::*;

/// Contents of an entry in a `#[derive_deftly(..)]` attribute
enum InvocationEntry {
    Precanned(syn::Path, UnprocessedOptions),
}

// (CannedName, CannedName, ...)
struct InvocationAttr {
    entries: Punctuated<InvocationEntry, token::Comma>,
}

/// Contents of an entry in a `#[derive_deftly_adhoc(..)]` attribute
#[derive(Default)]
struct AdhocAttr {
    pub_: Option<MacroExport>,
}

impl Parse for InvocationEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let entry = if input.lookahead1().peek(Token![pub]) {
            return Err(input.error("`pub` must be in #[derive_deftly_adhoc]"));
        } else {
            let path = syn::Path::parse_mod_style(input)?;
            let options = if input.peek(syn::token::Bracket) {
                let tokens;
                let _bracket = bracketed!(tokens in input);
                UnprocessedOptions::parse(
                    &tokens,
                    OpContext::DriverApplicationCapture,
                )?
            } else {
                UnprocessedOptions::default()
            };
            InvocationEntry::Precanned(path, options)
        };
        Ok(entry)
    }
}

impl Parse for InvocationAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let entries = Punctuated::parse_terminated(input)?;
        Ok(InvocationAttr { entries })
    }
}

fn check_for_misplaced_atrs(data: &syn::Data) -> syn::Result<()> {
    let attrs = |attrs: &[syn::Attribute]| {
        for attr in attrs {
            if let Some(_) = ["derive_deftly", "derive_deftly_adhoc"]
                .iter()
                .find(|forbidden| attr.path().is_ident(forbidden))
            {
                return Err(attr.error(
 "attribute is only meaningful at the data structure toplevel"
                ));
            }
        }
        Ok(())
    };

    let fields = |fs: &Punctuated<syn::Field, _>| {
        for f in fs.iter() {
            attrs(&f.attrs)?;
        }
        Ok(())
    };

    let variantish = |fs: &syn::Fields| match fs {
        syn::Fields::Unit => Ok(()),
        syn::Fields::Named(n) => fields(&n.named),
        syn::Fields::Unnamed(u) => fields(&u.unnamed),
    };

    let variants = |vs: &Punctuated<syn::Variant, _>| {
        for v in vs.iter() {
            attrs(&v.attrs)?;
            variantish(&v.fields)?;
        }
        Ok(())
    };

    match data {
        syn::Data::Struct(s) => variantish(&s.fields),
        syn::Data::Union(u) => fields(&u.fields.named),
        syn::Data::Enum(e) => variants(&e.variants),
    }
}

/// Returns the template macro name, for a given template name (as a path)
fn templ_mac_name(mut templ_path: syn::Path) -> syn::Result<syn::Path> {
    if templ_path.segments.is_empty() {
        return Err(templ_path
            .leading_colon
            .as_ref()
            .expect("path with no tokens!")
            .error("cannot derive_deftly the empty path!"));
    }
    let last = templ_path.segments.last_mut().expect("became empty!");
    let name = TemplateName::try_from(last.ident.clone())?;
    last.ident = name.macro_name();
    Ok(templ_path)
}

/// This is #[derive(Deftly)]
pub fn derive_deftly(
    driver_stream: TokenStream,
) -> Result<TokenStream, syn::Error> {
    use engine::ChainNext;

    let driver: syn::DeriveInput = syn::parse2(driver_stream.clone())?;

    dprint_block!(&driver_stream, "#[derive(Deftly)] input");

    let driver_mac_name =
        format_ident!("derive_deftly_driver_{}", &driver.ident);

    let precanned_paths: Vec<(syn::Path, UnprocessedOptions)> = driver
        .attrs
        .iter()
        .map(|attr| {
            if !attr.path().is_ident("derive_deftly") {
                return Ok(None);
            }
            let InvocationAttr { entries } = attr.parse_in_parens()?;
            Ok(Some(entries))
        })
        .flatten_ok()
        .flatten_ok()
        .filter_map(|entry| match entry {
            Err(e) => Some(Err(e)),
            Ok(InvocationEntry::Precanned(path, options)) => {
                Some(Ok((path, options)))
            }
        })
        .collect::<syn::Result<Vec<_>>>()?;

    let adhoc: Option<AdhocAttr> = driver
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("derive_deftly_adhoc"))
        .inspect(|_: &&syn::Attribute| ())
        .map(|attr| {
            let adhoc = match &attr.meta {
                syn::Meta::Path(_) => AdhocAttr { pub_: None },
                syn::Meta::NameValue(nv) => {
                    return Err(nv
                        .eq_token
                        .error("arguments (if any) must be in parens"))
                }
                syn::Meta::List(syn::MetaList {
                    path: _,
                    delimiter,
                    tokens,
                }) => {
                    match delimiter {
                        syn::MacroDelimiter::Paren(_) => Ok(()),
                        syn::MacroDelimiter::Brace(t) => Err(t.span),
                        syn::MacroDelimiter::Bracket(t) => Err(t.span),
                    }
                    .map_err(|span| span.error("expected parentheses"))?;
                    let pub_ = Parser::parse2(
                        MacroExport::parse_option,
                        tokens.clone(),
                    )?;
                    AdhocAttr { pub_ }
                }
            };
            Ok::<AdhocAttr, syn::Error>(adhoc)
        })
        .inspect(|_: &Result<AdhocAttr, _>| ())
        // allow this attr to be repeated; any pub makes it pub
        .reduce(|a, b| {
            let pub_ = chain!(a?.pub_, b?.pub_).next();
            Ok(AdhocAttr { pub_ })
        })
        .transpose()?;

    check_for_misplaced_atrs(&driver.data)?;

    let engine_macro = engine_macro_name()?;

    // If the driver contains any $ tokens, we must do something about them.
    // Otherwise, they might get mangled by the macro_rules expander.
    // In particular, the following cause trouble:
    //   `$template`, `$passthrough` - taken as references to the
    //      macro arguments.
    //  `$$` - taken as a reference to the nightly `$$` macro rules feature
    //     (which we would love to use here, but can't yet)
    //
    // `$orig_dollar` is a literal dollar which comes from the driver
    // invocation in invocation.rs.  This technique doesn't get the span
    // right.  But getting the span right here is hard without having
    // a whole new quoting scheme - see the discussion in the doc comment
    // for `escape_dollars`.
    //
    // We can't use the technique we use for the template, because that
    // technique relies on the fact that it's *us* that parses the template.
    // But the driver is parsed for us by `syn`.
    //
    // Actual `$` in drivers will be very rare.  They could only appear in
    // attributes or the like.  So, unlike with templates (which are
    // full of important `$`s) we can probably live with the wrong spans.
    let driver_escaped = escape_dollars(driver_stream);

    let mut output = TokenStream::new();

    let mut accum_start = TokenStream::new();

    if let Some(adhoc) = adhoc {
        accum_start.extend(quote!( _meta_used * ));

        let macro_export = adhoc
            .pub_
            .map(|export| {
                let macro_export =
                    quote_spanned!(export.span()=> #[macro_export]);
                Ok::<_, syn::Error>(macro_export)
            })
            .transpose()?;

        output.extend(quote! {
            #[allow(unused_macros)]
            #macro_export
            macro_rules! #driver_mac_name {
                {
                    { $($template:tt)* }
                    { ($orig_dollar:tt) $(future:tt)* }
                    $($dpassthrough:tt)*
                } => {
                    #engine_macro!{
                        { #driver_escaped }
                        ( )
                        { $($template)* }
                        $($dpassthrough)*
                    }
                };
                { $($wrong:tt)* } => {
                    compile_error!{concat!(
                        "wrong input to derive-deftly driver inner macro ",
                        stringify!(#driver_mac_name),
                     "; might be due to incompatible derive-deftly versions(s)",
                    )}
                };
            }
        });
    }

    let (chain_next, chain_rest);
    {
        let mut errs = ErrorAccumulator::default();
        let mut chain = chain!(
            precanned_paths
                .into_iter()
                .map(|(templ_path, aoptions)| {
                    let call = templ_mac_name(templ_path)?.to_token_stream();
                    let ao_versions = OpCompatVersions::ours();
                    let after_driver = quote!( [ #ao_versions #aoptions ] () );
                    Ok(ChainNext { call, after_driver })
                })
                .filter_map(|r| errs.handle(r)),
            [ChainNext {
                call: engine_macro.clone(),
                after_driver: quote!( . () ),
            }],
        );
        chain_next = chain.next().expect("should have been nonempty!");
        chain_rest = {
            let mut rest = TokenStream::new();
            for c in chain {
                c.to_tokens(&mut rest);
            }
            rest
        };

        errs.finish()?;
    }
    let ChainNext { call, after_driver } = chain_next;

    output.extend(quote! {
        #call !{
            { #driver }
            #after_driver
            [ #chain_rest ]
            [ #accum_start ]
        }
    });

    dprint_block!(&output, "#[derive(Deftly)] output for {}", &driver.ident);

    Ok(output)
}
