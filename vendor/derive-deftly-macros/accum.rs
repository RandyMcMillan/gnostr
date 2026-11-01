//! `derive_deftly_engine!()`, parsing accumulations

use super::framework::*;
use adviseable::*;

/// `derive_deftly_engine! accumulated form, accumulated information
///
/// We don't reify the whole input;
/// instead, we accumulate directly in the `Parse` impl.
#[derive(Debug)]
pub struct EngineFinalInput {
    driver: syn::DeriveInput,
    accum: Accumulated,
}

#[derive(Debug)]
pub struct Accumulated {
    metas: meta::CheckUsed<meta::Accum>,
}

impl EngineFinalInput {
    pub fn parse_adviseable_remainder(
        driver: syn::DeriveInput,
        input: ParseStream,
    ) -> AdviseableResult<Self> {
        let _empty_next_brackets_contents;
        let _ = bracketed!(_empty_next_brackets_contents in input);

        let accum;
        let _ = bracketed!(accum in input);
        let accum = accum.parse()?;

        let _: TokenStream = input.parse()?;

        Ok(AOk(EngineFinalInput { driver, accum }))
    }
}

impl Parse for Accumulated {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        use meta::CheckUsed as mCU;

        let mut metas = mCU::Check(meta::Accum::default());

        struct Ignore;

        while !input.is_empty() {
            let kind: syn::Ident = input.parse()?;
            match if kind == "_meta_used" {
                if let mCU::Check(m) = &mut metas {
                    match input.parse()? {
                        mCU::Check(y) => m.used.push(y),
                        mCU::Unchecked => metas = mCU::Unchecked,
                    }
                    continue;
                } else {
                    Ignore
                }
            } else if kind == "_meta_recog" {
                if let mCU::Check(m) = &mut metas {
                    let content;
                    let _brackets = bracketed!(content in input);
                    let input = content;
                    while !input.is_empty() {
                        use meta::Usage as mU;
                        let allow = match input.parse()? {
                            Some::<Token![?]>(_) => mU::BoolOnly,
                            None => mU::Value,
                        };
                        let desig = input.parse()?;
                        m.recog.update(desig, allow);
                    }
                    continue;
                } else {
                    Ignore
                }
            } else if kind == "error" {
                metas = mCU::Unchecked;
                Ignore
            } else if kind.to_string().starts_with('_') {
                Ignore
            } else {
                return Err(
                    kind.error("unrecognised mandatory accumulation kind")
                );
            } {
                Ignore => {
                    let _: TokenTree = input.parse()?;
                }
            }
        }
        Ok(Accumulated { metas })
    }
}

impl EngineFinalInput {
    pub fn process(self) -> syn::Result<TokenStream> {
        let r = Context::call(
            &self.driver,
            &dummy_path(), // template_crate, not used by our f
            None,          // template_name
            |ctx| {
                if let mCU::Check(m) = &self.accum.metas {
                    for group in &m.used {
                        adviseable_parse2_call(
                            group.content.clone(),
                            |input| {
                                ctx.decode_update_metas_used(input)?;
                                Ok(AOk(()))
                            },
                        )?
                    }
                }

                let mut errors = ErrorAccumulator::default();

                if let mCU::Check(m) = &self.accum.metas {
                    ctx.check_metas_used(&mut errors, &m.recog);
                }
                errors.finish()
            },
        );

        let mut out = TokenStream::new();

        match r {
            Ok(()) => {}
            Err(e) => e.into_compile_error().to_tokens(&mut out),
        }

        Ok(out)
    }
}
