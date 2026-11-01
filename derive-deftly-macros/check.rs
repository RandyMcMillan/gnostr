//! Implementation of the `expect` option

use crate::prelude::*;

/// Value for an `expect`
#[derive(Debug, Clone, Copy, Eq, PartialEq, EnumString, Display)]
#[allow(non_camel_case_types)]
pub enum Target {
    items,
    expr,
}

/// Local context for a syntax check operation
struct Checking<'t> {
    ctx: &'t framework::Context<'t>,
    output: &'t mut TokenStream,
    target: DdOptVal<Target>,
}

/// Main entrypoint
///
/// Checks that `output` can be parsed as `target`.
///
/// If not, replaces `output` with something which will generate
/// compiler error(s) which the user will find helpful:
///  * A `compile_error!` invocation with the original error span
///  * include_file!` for a generated temporary file
///    containing the text of the output,
///    so that the compiler will point to the actual error.
pub fn check_expected_target_syntax(
    ctx: &framework::Context,
    output: &mut TokenStream,
    target: DdOptVal<Target>,
) {
    check::Checking {
        ctx,
        output,
        target,
    }
    .check();
}

pub fn check_expect_opcontext(
    op: &DdOptVal<Target>,
    context: OpContext,
) -> syn::Result<()> {
    use OpContext as OC;
    match (context, op.value) {
        (OC::TemplateDefinition, Target::items) => Ok(()),
        (OC::TemplateDefinition, _) => {
            Err(op.span.error(
                "predefined templates must always expand to items", //
            ))
        }
        _ => Ok(()),
    }
}

impl Target {
    /// Checks if `ts` can parse as `self`, returning the error if not
    fn perform_check(self, ts: TokenStream) -> Option<syn::Error> {
        fn chk<T: Parse>(ts: TokenStream) -> Option<syn::Error> {
            syn::parse2::<Discard<T>>(ts).err()
        }

        use Target::*;
        match self {
            items => chk::<Concatenated<Discard<syn::Item>>>(ts),
            expr => chk::<syn::Expr>(ts),
        }
    }

    /// Tokens for `include!...` to include syntax element(s) like `self`
    fn include_syntax(self, file: &str) -> TokenStream {
        use Target::*;
        match self {
            items => quote! { include!{ #file } },
            expr => quote! { include!( #file ) },
        }
    }

    /// Make a single output, syntactically a `self.target`, out of pieces
    ///
    /// `err` is a `compile_error!` call,
    /// and `expansion` is typically the template expansion output.
    fn combine_outputs(
        self,
        mut err: TokenStream,
        expansion: TokenStream,
    ) -> TokenStream {
        use Target::*;
        match self {
            items => {
                err.extend(expansion);
                err
            }
            expr => quote!( ( #err, #expansion ) ),
        }
    }
}

impl Checking<'_> {
    /// Checks that `tokens` can be parsed as `T`
    ///
    /// Does the actual work of [`check_expected_target_syntax`]
    fn check(self) {
        let err = self.target.value.perform_check(self.output.clone());

        let err = match err {
            Some(err) => err,
            None => return,
        };

        let broken = mem::take(self.output);
        let err = err.into_compile_error();

        let expansion = expand_via_file(self.ctx, self.target.value, broken)
            .map_err(|e| {
                Span::call_site()
                    .error(format!(
 "derive-deftly was unable to write out the expansion to a file for fuller syntax error reporting: {}",
                    e
                ))
                    .into_compile_error()
            })
            .unwrap_or_else(|e| e);

        *self.output = self.target.value.combine_outputs(err, expansion);
    }
}

/// Constructs an `include!` which includes the text for `broken`
///
/// Appends the `include` to `checking.output`.
///
/// If this can't be done, reports why not.
fn expand_via_file(
    ctx: &framework::Context,
    target: Target,
    broken: TokenStream,
) -> Result<TokenStream, String> {
    use sha3::{Digest as _, Sha3_256};
    use std::{fs, io, io::Write as _, path::PathBuf};

    let text = format!(
        "// {}, should have been {}:\n{}\n",
        ctx.expansion_description(),
        target,
        broken,
    );

    let hash: String = {
        let mut hasher = Sha3_256::new();
        hasher.update(&text);
        let hash = hasher.finalize();
        const HASH_LEN_BYTES: usize = 12;
        hash[0..HASH_LEN_BYTES].iter().fold(
            String::with_capacity(HASH_LEN_BYTES * 2),
            |mut s, b| {
                write!(s, "{:02x}", b).expect("write to String failed");
                s
            },
        )
    };

    let dir: PathBuf = [env!("OUT_DIR"), "derive-deftly~expansions~"]
        .iter()
        .collect();

    match fs::create_dir(&dir) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
        Err(e) => return Err(format!("create dir {:?}: {}", &dir, e)),
    };

    let leaf = format!("dd-{}.rs", hash);
    let some_file = |leaf: &str| {
        let mut file = dir.clone();
        file.push(leaf);
        file
    };
    let file = some_file(&leaf);
    let file = file
        .to_str()
        .ok_or_else(|| format!("non UTF-8 path? from env var! {:?}", file))?;

    // We *overwrite* the file in place.
    //
    // This is because it's theoretically possible that multiple calls
    // to this function, at the same time, might be generating files
    // with identical contents, and therefore the same name.
    //
    // So we open it with O_CREATE|O_WRITE but *not* O_TRUNC,
    // and write our data, and then declare our job done.
    // This is idempotent and concurrency-safe.
    //
    // There is no need to truncate the file, since all writers
    // are writing the same text.  (If we change the hashing scheme,
    // we must change the filename too.)

    let mut fh = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(file)
        .map_err(|e| format!("create/open {:?}: {}", &file, e))?;
    fh.write_all(text.as_ref())
        .map_err(|e| format!("write {:?}: {}", &file, e))?;

    Ok(target.include_syntax(file))
}
