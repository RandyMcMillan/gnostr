//! Implementation of `${approx_equal ..}`, and support functions
//
// # Note on error handling
//
// Many functions here take `cmp_loc: ErrorLoc` and return `syn::Result`.
// `cmp_loc` is the comparison operator (`kw_span` in `boolean.rs`,
// referring to the `approx_equal` keyword.
//
// When generating errors, we include this in our list of ErrorLocs.
//
// An alternative would be to return a bespoke error type,
// consisting of the pieces to make the error from.
// I experimented with this, but it's definitely worse.
// Also this has trouble handling a `syn::Error` from other code we call.

use super::prelude::*;

use proc_macro2::Group;
use Equality::*;

/// Return value of a (perhaps approximate) equality comparison
///
/// (Avoids use of `bool`)
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Equality {
    Equal,
    Different,
}

impl Equality {
    /// Compare `a` and `b`
    ///
    /// (Name is short but avoids clash with `Ord::cmp`)
    pub fn cmpeq<T: Eq>(a: &T, b: &T) -> Self {
        if a == b {
            Equal
        } else {
            Different
        }
    }
}

/// Compare, and return early if different
///
///  * **`cmpeq!(d: Equality)`**:
///    If `d` is `Different`, returns `Ok(d)`.
///    (The containing scope should return `Result<Equality,>`.)
///
///  * **`cmpeq!<T: Eq>(a: T, b: T);`**:
///    compares `a` and `b` using `Equality::cmpeq`,
///    and returns immediately if `a != b`,
///    or the comparison failed.
macro_rules! cmpeq {
    { $a:expr, $b:expr } => {
        cmpeq!(Equality::cmpeq(&$a, &$b));
    };
    { $r:expr } => {
        if let d @ Different = $r {
            return Ok(d);
        }
    };
}

/// Return the input, but with `None`-delimited `Group`s flattened away
///
/// Loses some span information.
pub fn flatten_none_groups(ts: TokenStream) -> TokenStream {
    fn recurse(out: &mut TokenStream, input: TokenStream) {
        for tt in input {
            match tt {
                TT::Group(g) if g.delimiter() == Delimiter::None => {
                    recurse(out, g.stream());
                }
                TT::Group(g) => {
                    let span = g.span();
                    let mut g = Group::new(
                        g.delimiter(),
                        flatten_none_groups(g.stream()),
                    );
                    // We lose some span information here.
                    g.set_span(span);
                    out.extend([TT::Group(g)]);
                }
                _ => out.extend([tt]),
            }
        }
    }

    let mut out = TokenStream::new();
    recurse(&mut out, ts);
    out
}

trait LitComparable {
    fn lc_compare(
        a: &Self,
        b: &Self,
        cmp_loc: &ErrorLoc<'_>,
    ) -> syn::Result<Equality>;
}
trait LitConvertible {
    type V: Eq;
    fn lc_convert(&self, cmp_loc: &ErrorLoc<'_>) -> syn::Result<Self::V>;
}
fn str_check_suffix(
    suffix: &str,
    span: Span,
    cmp_loc: &ErrorLoc<'_>,
) -> syn::Result<()> {
    if suffix.is_empty() {
        Ok(())
    } else {
        Err([(span, "literal"), *cmp_loc].error(
 "comparison of string/byte/character literals with suffixes is not supported"
        ))
    }
}
macro_rules! impl_LitComparable_str { { $lit:ty, $val:ty } => {
    impl LitConvertible for $lit {
        type V = $val;
        fn lc_convert(&self, cmp_loc: &ErrorLoc<'_>) -> syn::Result<Self::V> {
            str_check_suffix(self.suffix(), self.span(), cmp_loc)?;
            Ok(self.value())
        }
    }
} }

impl_LitComparable_str!(syn::LitStr, String);
impl_LitComparable_str!(syn::LitByteStr, Vec<u8>);
impl_LitComparable_str!(syn::LitByte, u8);
impl_LitComparable_str!(syn::LitChar, char);

impl<T: LitConvertible> LitComparable for T {
    fn lc_compare(
        a: &Self,
        b: &Self,
        cmp_loc: &ErrorLoc<'_>,
    ) -> syn::Result<Equality> {
        Ok(Equality::cmpeq(
            //
            &a.lc_convert(cmp_loc)?,
            &b.lc_convert(cmp_loc)?,
        ))
    }
}

impl LitConvertible for syn::LitBool {
    type V = ();
    fn lc_convert(&self, _cmp_loc: &ErrorLoc<'_>) -> syn::Result<Self::V> {
        Err(self.error(
            "internal error - TokenTree::Literal parsed as syn::Lit::Bool",
        ))
    }
}

impl LitConvertible for syn::LitFloat {
    type V = String;
    fn lc_convert(&self, _cmp_loc: &ErrorLoc<'_>) -> syn::Result<Self::V> {
        Ok(self.token().to_string())
    }
}

impl LitComparable for syn::LitInt {
    fn lc_compare(
        a: &Self,
        b: &Self,
        cmp_loc: &ErrorLoc<'_>,
    ) -> syn::Result<Equality> {
        match (
            a.base10_parse::<u64>(),
            b.base10_parse::<u64>(),
        ) {
            (Ok(a), Ok(b)) => Ok(Equality::cmpeq(&a, &b)),
            (Err(ae), Err(be)) => Err(
                [(a.span(), &*format!("left: {}", ae)),
                 (b.span(), &*format!("right: {}", be)),
                 *cmp_loc,
                ].error(
 "integer literal comparison with both values >u64 is not supported"
                )),
            (Err(_), Ok(_)) | (Ok(_), Err(_)) => Ok(Different),
        }
    }
}

fn lit_cmpeq(
    a: &TokenTree,
    b: &TokenTree,
    cmp_loc: &ErrorLoc<'_>,
) -> syn::Result<Equality> {
    let mk_lit = |tt: &TokenTree| -> syn::Result<syn::Lit> {
        syn::parse2(tt.clone().into())
    };

    let a = mk_lit(a)?;
    let b = mk_lit(b)?;

    syn_lit_cmpeq_approx(a, b, cmp_loc)
}

/// Compare two literals the way `approx_equal` does
///
/// `pub` just so that the tests in `directly.rs` can call it
pub fn syn_lit_cmpeq_approx(
    a: syn::Lit,
    b: syn::Lit,
    cmp_loc: &ErrorLoc<'_>,
) -> syn::Result<Equality> {
    macro_rules! match_lits { { $( $V:ident )* } => {
        let mut error_locs = vec![];
        for (lit, lr) in [(&a, "left"), (&b, "right")] {
            match lit {
              $(
                syn::Lit::$V(_) => {}
              )*
                _ => error_locs.push((lit.span(), lr)),
            }
        }
        if !error_locs.is_empty() {
            return Err(error_locs.error(
                "unsupported literal(s) in approx_equal comparison"
            ));
        }

        match (&a, &b) {
          $(
            (syn::Lit::$V(a), syn::Lit::$V(b))
                  => LitComparable::lc_compare(a, b, cmp_loc),
          )*
            _ => Ok(Different),
        }
    } }

    // We do not support comparison of `CStr`.
    // c"..." literals are recognised only by Rust 1.77,
    // and we would need syn 2.0.59 to parse them.
    // So this would require
    //   - bumping our syn dependency to 2.0.59 globally,
    //     or somehow making that feature-conditional,
    //     or messing about parsing the lockfile in build.rs.
    //   - Adding an MSRV-influencing feature,
    //     or testing the rustc version in build.rs.
    // I hoping we can put this off.

    match_lits! {
        Str
        ByteStr
        Byte
        Char
        Bool
        Int
        Float
    }
}

fn tt_cmpeq(
    a: TokenTree,
    b: TokenTree,
    cmp_loc: &ErrorLoc<'_>,
) -> syn::Result<Equality> {
    let discrim = |tt: &_| match tt {
        TT::Punct(_) => 0,
        TT::Literal(_) => 1,
        TT::Ident(_) => 2,
        TT::Group(_) => 3,
    };

    cmpeq!(discrim(&a), discrim(&b));
    match (a, b) {
        (TT::Group(a), TT::Group(b)) => group_cmpeq(a, b, cmp_loc),
        (a @ TT::Literal(_), b @ TT::Literal(_)) => lit_cmpeq(&a, &b, cmp_loc),
        (a, b) => Ok(Equality::cmpeq(&a.to_string(), &b.to_string())),
    }
}

fn group_cmpeq(
    a: Group,
    b: Group,
    cmp_loc: &ErrorLoc<'_>,
) -> syn::Result<Equality> {
    let delim =
        |g: &Group| Group::new(g.delimiter(), TokenStream::new()).to_string();
    cmpeq!(delim(&a), delim(&b));
    ts_cmpeq(a.stream(), b.stream(), cmp_loc)
}

/// Internal, recursive, comparison of flattened `TokenStream`s
fn ts_cmpeq(
    a: TokenStream,
    b: TokenStream,
    cmp_loc: &ErrorLoc<'_>,
) -> syn::Result<Equality> {
    for ab in a.into_iter().zip_longest(b) {
        let (a, b) = match ab {
            EitherOrBoth::Both(a, b) => (a, b),
            EitherOrBoth::Left(_) => return Ok(Different),
            EitherOrBoth::Right(_) => return Ok(Different),
        };
        match tt_cmpeq(a, b, cmp_loc)? {
            Equal => {}
            neq => return Ok(neq),
        }
    }
    return Ok(Equal);
}

/// Compares two `TokenStream`s for "equivalence"
///
/// We intend that two `TokenStream`s count as "equivalent"
/// if they mean the same thing to the compiler,
/// modulo any differences in spans.
///
/// We also disregard spacing.  This is not 100% justifiable but
/// I think there are no token sequences differing only in spacing
/// which are *both* valid and which differ in meaning.
///
/// ### Why ?!
///
/// `< <` and `<<` demonstrate that it is not possible to provide
/// a fully correct and coherent equality function on Rust tokens,
/// without knowing the parsing context:
///
/// In places where `<<` is a shift operator, `< <` is not legal.
/// But in places where `<<` introduces two lots of generics,
/// `<<` means the same.
///
/// I think a function which treats `< <` and `<<` as equal is more useful
/// than one that doesn't, because it will DTRT for types.
///
/// ### `None`-delimited `Group`s
///
/// We flatten these
///
/// This is necessary, because otherwise
/// apparently-identical pieces of code count as different.
///
/// This does mean that two things which are `approx_equal`
/// can be expressions with different values!
///
/// But, the Rust grammar for types doesn't permit ambiguity,
/// so the type equality guarantee of `approx_equal` is preserved.
//
// Comparing for equality has to be done by steam.
// And a lot of stringification.
pub fn tokens_cmpeq(
    a: TokenStream,
    b: TokenStream,
    cmp_span: Span,
) -> syn::Result<Equality> {
    let a = flatten_none_groups(a);
    let b = flatten_none_groups(b);
    ts_cmpeq(a, b, &(cmp_span, "comparison"))
}
