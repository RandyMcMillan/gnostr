# Changelog, MSRV policy and cargo features

<!-- @dd-navbar toc .. -->
<!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="../index.html">crate top-level</a> | <a href="../index.html#overall-toc"><strong>overall toc</strong>, macros</a> | <a href="../doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/">guide/tutorial</a></em> ]</nav>

<div id="t:cargo-features">

## cargo features

</div>

Features are provided to allow for a build with reduced dependencies.

 * `full`: Metafeature.
   Enable all reasonable, non-experimental, features.
 * `full-msrv-1.56`: Metafeature.
   Enable all reasonable, non-experimental, features
   compatible with Rust 1.56.
 * `case`: [Case conversions](../doc_reference/index.html#case-changing),
   using `heck`.
 * `expect`: the [`expect`](../doc_reference/index.html#expect-items-expect-expr--syntax-check-the-expansion)
   expansion option for syntax checking.
 * `meta-as-expr`, `meta-as-items`:
   handling attributes containing (syntax-checked) expressions and items via
   [`${Xmeta(...) as expr / items}`](../doc_reference/index.html#tmeta-vmeta-fmeta--deftly-attributes)
 * `minimal-1`: Minimal feature set.  Must be enabled.
 * <span id="cargo-feature:beta">`beta`</span>: Enable
   [beta features](#t:beta).

All of the above features except `beta` are enabled by default.

## MSRV and MSRV policy

The Minimum Supported Rust Version for derive-deftly is 1.56.

We expect to increase it cautiously and only with good reason.
(However, MSRV increase would be a minor version bump.)

Future cargo features might imply a higher MSRV.
Such features might be added to the metafeature `full`,
again, in a minor version bump.

## Reporting issues

We can't wait to hear from you!

In order to submit a bug to
[the derive-deftly issue tracker][tracker]
you will need an account on <https://gitlab.torproject.org>.

You can request an account via
[anonticket](https://anonticket.torproject.org/),
or via the [account request form].

[tracker]: https://gitlab.torproject.org/Diziet/rust-derive-deftly/-/issues
[account request form]: https://anonticket.torproject.org/user/gitlab-account/create/

## Changelog

### 0.14.6

#### Improved

 * docs: Fixed some version numbers in version-specific xref links.

### 0.14.5

#### Improved

 * Improvements to documentation cross-references and linking.
 * Relaxed upper dependency bounds for `strum`.

### 0.14.4

#### Fixed

 * Removed ill-nested divs from README.md.

### 0.14.3

#### Added

 * `beta` cargo feature, for unstable derive-deftly features.
 * BETA: `${Xmeta ... default ...}`.

#### Improved

 * docs: Improvements to the Guide.
 * docs: Other documentation improvements.
 * docs: New experimental navbars for traversing derive-deftly docs.
 * Relaxed upper dependency bounds for `educe`, `itertools`.
 * Internal and testing improvements.

### 0.14.2

#### Improved

 * Improvements to the Guide.
 * New "Reporting issues" docs, currently in CHANGELOG.md.

### 0.14.1

#### Fixed

 * Failing `expect expr` now successfully points to the error in
   a copy of the template expansion, as intended.

#### Improved

 * Reference now has indices of expansion and condition keywords.
 * Substantial improvements and additions to the Guide.
 * Inproved `dbg_all_keywords`; now includes user defined expansions.
 * Various new `<div>` anchors in the Reference and this changelog.
 * Many internal and testing improvements.

### 0.14.0

#### Breaking

 * Desupport syntax deprecated in 0.12.x:
   - Rqquire `define_derive_deftly! { Template: ... }`,
     no longer tolerating `define_derive_deftly! { Template = ... }`.
   - Require `export` to export templates (and drivers),
     no longer tolerating `pub`.
	 Eg `define_derive_deftly! { export Template: ... }`.
   - Remove `pub_template_semver_check!` macro,
     in favour of `template_export_semver_check!`.

#### Improved

 * README now hopes that we've had the last breaking changes before 1.0.
 * Old syntax is no longer found in various places in the documentation.

#### template export semver

 * Versions before 0.12.1 are rejected by `template_export_semver_check!`:
   Compatibility with them is no longer testable in CI, and may break.
   (You may be able to keep using older versions, by removing the
   call to `template_export_semver_check!`, but if so you're on your own.)

   Crates that export templates, and haven't bumped their own semver
   since before derive-deftly 0.12.1, should probably bump semver now.

### 0.13.1

#### Fixed

 * Fixed MSRV violation introduced in 0.13.0.
   (cargo fails with "error: unsupported output in build script".)
   See [#103](https://gitlab.torproject.org/Diziet/rust-derive-deftly/-/issues/103).

#### Improved

 * Declare `rust-version` in `derive-deftly-macros`'s `Cargo.toml`, too.

### 0.13.0

#### Breaking

 * `${approx_equal }` compares literals (except floats) by value.
   It can now fail (giving an error) if it can't do the comparison.
 * `${when }`'s must now come before anything else in their repetition,
   and this is checked when parsing so enforced even if not expanded.
 * `${Xmeta as expr}` is now behind a new `meta-as-expr` feature,
   which is enabled by default but not part of `minimal-1`.

#### Added

 * New `full-msrv-1.56` feature.  `full` may increase MSRV more quickly.
 * Provide a [stability policy](#t:stability).

#### Fixed

 * Pasting now works correctly when some of the ingredients are
   raw identifiers.

#### Improved

 * Compatibility with Rust's new
   [cfg checking scheme](https://blog.rust-lang.org/2024/05/06/check-cfg.html).
 * Improvements and fixes to reference documentation.
 * Many improvements to internals and tests.

### 0.12.1

#### Deprecated

 * Deprecate `pub` (in template definitions and driver adhoc exports)
   in favour of `export`; and deprecate `pub_template_semver_check!`
   in favour of `template_export_semver_check!`.
   `pub` etc. are still supported for now, but will be removed soon.

### 0.12.0

#### Breaking

 * Forbid defining templates named starting with lowercase letters.
   (This is now reserved for future expansion.)

#### Deprecated

 * `define_derive_deftly!` calls should use `:` now, rather tha `=`.
   `=` is still supported, for now, but it will be removed soon.

#### Added

 * Accept ":" in `define_derive_deftly!`, as well as "=",
 * Reference has stable anchors (suitable for use with `#...` in URLs)
   for each keyword and section.

#### Improved

 * Guide (previously the Tutorial aka Introduction) is now an mdbook book.
   (Links from out-of-tree may need updating.)
 * Many improvements to the Guide.

### 0.11.0

#### Breaking

 * `${Xmeta as ...}` no longer has a default.
   Change to `as expr`, `as ty`, etc.
 * `${Xmeta as tokens}` is now `as token_stream`
   (and we now document its inherent precedence hazards).
 * `${Xmeta as ty}` and `$ftype` insert `::` before generic arguments
   (ie, they normalise the turbofish);
   this is generally breaking only with `${approx_equal ..}`.
 * `None`-delimited groups are ignored by `${approx_equal ..}`.
 * Meta attributes `#[deftly]` must actually be used in
   an expanded `$tmeta`/`$vmeta`/`$fmeta` expansion,
   or (as applicable),
   tested with a `tmeta`/`vmeta`/`fmeta` condition.
   I.e. meta attribute use checking is dynamic:
   merely mentioning the meta somewhere in an unused part of the template
   is not enough.

#### Added

 * `${Xmeta as expr/ident/path/items}`
 * `meta-as-items` cargo feature (for `${Xmeta as items}`),
   part of `full` and enabled by default.
 * Parenthesised type path attributes, processed by `${Xmeta as ty}`,
   can now be pasted with `${paste }`.

#### Improved

 * `${Xmeta as ty}` uses a `None`-delimited `Group`
   (although, this is not effective with current compilers:
   [rust-lang/rust#67062](https://github.com/rust-lang/rust/issues/67062).
 * Precedence considerations with `${define }` and `${Xmeta }`
   are documented in the reference.
 * Short-circuiting behaviour of `any` and `all` conditions is documented.
 * Improved some error messages.

#### template export semver

Updating to derive-deftly 0.11.0 is a breaking change
to the API of a crate which exports templates:

The scheme for collating information about meta attributes,
across multiple template invocations, has been completely changed.
So the calling convention for the template macro is quite incompatible.

### 0.10.5

#### Breaking (minor)

 * Fixed `$vdefbody` to always include precisely a comma after enum
   variants.  The previous wrong behaviour was shown in the example output,
   but wasn't usable in the documented/intended manner.  \#63, !325.

#### Improved

 * Improved some error messages.
 * Docs clarifications and fixes.
 * Testing fixes and robustness improvements.

### 0.10.4

#### Added

 * `${ignore }` keyword.

#### Fixed

 * Avoid spurious "unexpected token" errors after other errors.

#### Improved

 * Documentation updates.
 * Slightly improved an error message.

### 0.10.3

#### Fixed

 * Fix `approx_equal` not to treat its arguments as equal
   if one of them was a prefix of the other.  (#52)

#### Breaking (minor)

 * Reject `expect expr` early, in `define_derive_deftly!`.
   (Such a template could never be successfully used.)

#### Added

 * `tgens` condition: true if there are any generics.
 * `${dbg ..}` debug dump expansion, and `dbg(...)` debug dump condition.
 * `is_empty` condition for testing an expansion for emptiness.

#### Improved

 * Improvements to reference docs.

### 0.10.2

#### Fixed

 * Docs references to `pub_template_semver_check` corrected
   (including in changelog entry for 0.10.0).

#### Improved

 * Use the 2021 edition.  Some minimum dependency requirements
   tightened, but: no MSRV impact.

### 0.10.1

#### Added

 * `${error "message"}` keyword for explicitly causing compile errors.

#### Improved

 * Prettier output from `${dbg_all_keywords}`.
 * Better error messages for unecognised `#[deftly(...)]` attributes.

### 0.10.0

#### Breaking

 * Reject invalid templates in `define_derive_deftly!`,
   even if they are never invoked.
 * Reject `#[deftly(...)]` attributes not recognised by
   any relevant template.

#### Added

 * `pub_template_semver_check!` for helping ensure semver
   compliance when exporting macros, and associated docs.

#### Fixed

 * Fixed a largely-theoretical concurrency bug which could cause
   wrong error output if multiple invocations generated identical,
   but syntactically-invalid, expansions.  (!262 / !263).
 * Improve spans in certain error messages relating to `Xmeta`.
 * Fix heading formatting issues in the entry for 0.9.0.

#### template export semver

Updating to derive-deftly 0.10.0 is a breaking change
to the API of a crate which exports templates:

Exported templates are not compatible due to renaming of the
internal keyword `$_da_intern_crate` to `$_dd_intern_crate`.

### 0.9.2

#### Improved

 * docs: Fixed the README.md cross-reference and stability information.

### 0.9.1

#### Improved

 * Documentation and error message fixes, tidying up after renames.
 * Other minor documentation improvements.

### 0.9.0

#### Breaking

 * Crate renamed to `derive-deftly`, and many other renamings:
 * `#[derive_deftly_adhoc]` is now required for `derive_deftly_adhoc!`
   (ie, non-precanned templates).
 * To export a driver, `#[derive_deftly_adhoc(pub)]` is now needed,
   rather than `#[derive_adhoc(pub)]`.

```text
Old name (0.8.x and earlier)	New name (0.9.x and later)

define_derive_adhoc!            define_derive_deftly!

#[derive(Adhoc)]                #[derive(Deftly)]
#[derive_adhoc(Template)]       #[derive_deftly(Template)]
#[adhoc(...)]                   #[deftly(...)]

derive_adhoc_driver!            derive_deftly_driver!
derive_adhoc_template!          derive_deftly_template!
derive_adhoc_expand!            derive_deftly_engine!

derive_adhoc!                   derive_deftly_adhoc!
```

##### template export semver

Updating to `derive-deftly` from `derive-adhoc` is a breaking change
to the API of a crate which exports templates.

### 0.8.1

#### Fixed

 * Fix doubled dollar deescaping in precanned d-a macros.
   Fixes `macro_rules! .. { { $$m:expr .. } ... }`.

#### Improved

 * Improvements to macro rustdoc documentation.
 * Minor internal improvements.

### 0.8.0

#### Nominally breaking (template export semver)

**We recommend you do *not* treat these changes as semver-breaking
for template-defining crates.**

 * Reject misplaced `#[derive_adhoc]` attributes.
 * Reject malformed attributes like `#[adhoc("42")]`
   and `#[adhoc(std::cell::<String>)]`.

Technically these are breaking changes, since users of exported
templates might have provided these wrong attributes, which were
previously ignored and are now properly rejected.  The attributes are
interpreted by the version of derive-adhoc which defines the template.
However, the likelihood of this breaking a working build seems low.

#### MSRV

 * MSRV increased from 1.54 to 1.56.
   Principally because we need this to update to syn 2.
   1.56 is the first release with Rust 2021, but
   we're not updating to the 2021 edition yet.

#### Fixed

 * Fixed a spurious clippy lint `crate_in_macro_def` (#27).
 * More reasonable handling of `#[adhoc(..)]` attributes whose meta
   information has paths some of which are nested sub-paths of others.
 * Work around a strange bug with literals and spans which can cause
   a `compiler/fallback mismatch` panic during error reporting
   with some compiler versions. (!182)

#### Improved

 * Docuemntation improvements (notably, many cross-references added
   from the tutorial to the reference).
 * Dependency updates (notably, we're now using syn 2).
 * Improvements to tests, CI, `HACKING.md`, Cargo lockfile handling.
 * Lockfile (`Cargo.lock`) is now committed to git under that name.

### 0.7.3

#### Added

 * `approx_equal` condition, for testing a form of token stream equivalence.

#### Fixed

 * docs: Corrected some internal links to refer to the correct anchors.

#### Improved

 * `$ttype`, `$tdeftype` and `$vtype` no longer include `::<>` or `<>`
   unless the original toplevel driver definition did.

### 0.7.2

#### Added

 * `${Xmeta(...)}` within pasting now defaults to `... as str`,
   so that doesn't need to be specified each time.

### 0.7.1

#### Fixed

 * Fix reference documentation examples for `$Xattr`.

### 0.7.0

#### Breaking

 * Reject inner attributes (`#![...]` and `//!...`) anywhere in templates:
   these are now reserve for future expansion.
   To resolve: use inner attributes instead.

#### Added

 * New `${define }` and `${defcond }` facility for reuseable template
   fragments and conditions, to save repetition in templates.  (#14)
 * Abbreviated `$<...>` syntax for `${paste ...}`.  (#23)

#### Fixed

 * Allow nested pasting when intermediate pastes aren't valid idents.
 * Fixed bugs relating to pasting of keywords and raw identifiers.
 * docs: Fixed minor bugs in the reference.

### 0.6.1

#### Fixed

 * Fixed a few broken docs links and similar infelicities.

#### Improved

 * Change dollar-escaping pseudo-keyword to `$orig_dollar`.
   (Internal change; should not have any user-visible effect.)

### 0.6.0

#### Breaking

 * `$fvis` and `fvis` now refer to the top-level visibility for enums.
   (The previous behaviour is now available from `$fdefvis`/`fdefvis`.)
 * `fmeta` and `vmeta` now fail when used outside a field or variant,
   rather than searching through the whole item.
   (`$fmeta` and `$vmeta` didn't search and are unchanged.)
 * Actually make structs and unions be treated as having one "variant".
 * `$tdefgens` and `$tgens` include trailing comma when nonempty,
   as documented and intended.

#### Added

 * `define_derive_adhoc!` supports doc comments.
 * `$fdefvis`/`fdefvis` for the textual visibility of a field.

#### Fixed

 * Reference doc example snippets: many errors corrected.
 * Added missing cargo dep on `syn/extra-traits`.

#### Improved

 * Trailing comma no longer added inside generics in `$ttype` `$vtype`.
 * Relaxed upper dependency bounds for `strum`, `itertools`.
 * docs: Reference: example snippets: now tested, and many added.
 * docs: Introduction: now has a Table of Contents.

### 0.5.0

#### Breaking

 * Case changing: non-snake-case keywords for case change instructions
   abolished (to make room for possible future reservation of
   non-snake-case keywords as user-defined ones).  Change the case
   of the keyword to snake case.

#### Improved

 * Documentation: tidying and a few more examples.

### 0.4.0

#### Breaking

 * `${paste }` no longer allows non-string literals in its content.
 * Invalid literal content within `${paste }` is now rejected
   during parsing, so even if that part of the template isn't expanded.
 * `${Xmeta... as lit}` is abolished, in favour of `... as str`
   (which has more consistent semantics).
 * `${Xmeta}` without `as` is no longer allowed in `${paste ...}`.
 * In `#[adhoc(attr="VALUE")]`, now only *string* literals are allowed.

#### Added

 * `${paste }` and `${CASE_CHANGE ...}` can now be nested,
    either (or both) ways round.
 * `${Xmeta... as str}`, `${Xmeta... as tokens}`.

#### Improved

 * Better error messages when `${paste }` produces a bad identifier.
 * docs: Minor improvements to reference.
 * internal: CI tests improved and extended
 * internal: cleanups, and internal docs improved.

### 0.3.0

#### Breaking

 * cargo features introduced.
   Currently, all enabled by default -
   no breakage if default features enabled.
   - Case conversion (and `heck` dependency) now behind `case`.
   - `minimal-1` introduced; it must be enabled.

#### Added

 * Expansion options facility
 * `expect items`, `expect expr` option, for improved errors
   when template expands to invalid Rust syntax.
   (`expect` cargo feature.)
 * `for struct`, `for enum`, `for union` option,
   for improved errors due to misue of a template.
 * `dbg` option, for dumping template expansions to compiler stderr.
 * `$dbg_all_keywords` keyword, for dumping keyword expansions.
 * `full` and `minimal-1` cargo meta features.

#### Improved

 * docs: Much expanded and improved tutorial (still a work in progress).
 * docs: Various corrections to reference docs.
 * docs: Reference documentation retitled and module renamed.
 * error handling: Better messages from certain situations involving
   multiple (incompatible) derive-adhoc versions.
 * tests: Made less fragile (more pinning of test dependencies).
 * tests: Improved CI checks on documentation, TODOs, etc.
 * internal: new HACKING.md file for helping develop derive-adhoc.

### 0.2.2

#### Fixed (future compatibility)

 * Pinned dependency from `derive-adhoc` to `derive-adhoc-macros`.
 * Handling of certain supposedly-future-compatible options fixed.
   ("future driver options" argument to `d_a_t_T`).

#### Improved

 * Better error messages with usupported combinations of features
   with mixed derive-adhoc versions.  #10.
 * Compatibility with derive-adhoc 0.2.0 tested in CI.

### 0.2.1

#### Fixed

 * `$vpat` expansion includes necessary post-field comma.  #15.
 * Docs typo fixes.

#### Improved

 * docs: Much expanded tutorial (still a work in progress)

### 0.2.0

#### Breaking

 * `$tgens` no longer includes defaults for generics.
 * `$Xattrs` by default
   outputs all attributes except derive-adhoc ones,
   rather than nothing (breaking bugfix).
 * `$vmeta` for a struct (not enum) processes top-level attributes,
   rather than imagining that there are no variant/value attributes.
 * Fixed hygiene (span) for `${paste }`;
   now it's consistently that of the template
   (but disagrees with the hygiene span of `$fname`).

#### Added

 * `$fpatname` `$vpat` `$vtype`, for value matching and construction
 * `$fvis` `$tvis`, for visibility (also as booleans)
 * `is_struct` `is_union`
   `v_is_unit` `v_is_tuple` `v_is_named`,
   conditions for driver shape.
 * `$tdefkwd` `$tdeftype` `$tdefvariants` `$vdefbody`
   `$fdefine` `$tdefgens`,
   for defining derived types,
 * `$select1`, exactly-one conditional
 * Support exporting a template to other crates,
   and `$crate` expansion for referring to template crate items.
 * Support exporting a driver to other crates
   (rather hazardous).

#### Fixed

 * Do not claim that `$ttype` includes any leading path elements.
 * `$` in templates always has proper span for error reporting
 * `$` is properly escaped in drivers
 * `$Xmeta` can expand to arbitrary tokens from an attribute value

#### Improved

 * docs: New tutorial (still a work in progress)
 * docs: Template syntax reference overhauled
 * Many other minor improvements
 * New tests/examples

### 0.1.0

 * First publicly advertised release.
   Much important documentation and
   many important features still missing.

### 0.0.1

 * Initial release to crates.io, not widely advertised.

<div id="t:stability">

## Stability and semver policy

</div>

We take a similar approach to stability to Rust upstream,
but we may make breaking changes, signaled via semver.

If you write your code according to a
reasonable interpretation of the documentation,
and it is accepted by the compiler,
we intend that it won't break with a new version of derive-deftly,
unless we have declared the breakage
by making a semver-breaking change to our version number.

Rarely, we may need to make a change which violates that rule,
especially if we discover a serious issue that might mean
actual programs in the wild are likely to be not behaving
as the programmer might expect.

When we say that something will be "rejected",
or use precisely the phrase "it is an error",
we intend that you can rely on that:
making it be accepted would be a breaking change.
In contrast, when we write that something is "not supported",
or use terminology like "must",
or show it with `error,` in examples
we might add support for it later.

There are additional, stronger, promises relating to the API
of exported templates,
to help you manage the semver implications for the exporting crate.
This is set out in the
[relevant documentation](../macro.define_derive_deftly.html#exporting-a-template-for-use-by-other-crates).

<div id="t:beta">

### Beta features

derive-deftly has some features which we are still experimenting with,
and whose syntax and semantics might change.

To use these, you must
enable the [`beta`](#cargo-feature:beta) cargo feature,
and,
pass the [`beta_deftly`](../doc_reference/index.html#eo:beta_deftly)
template option.

Beta features may
change their syntax or semantics or be removed,
without notice.
This would just be a patchlevel version bump,
(after 1.x, a minor version bump).
So you should pin your derive-deftly version:

```toml
// Cargo.toml
derive-deftly = { version = "=0.14.0", features = ["full", "beta"] }
// after derive-deftly 1.x,  { version = "~1.x.y" ... }
```

The `beta` cargo feature may imply a higher MSRV.

</div>
