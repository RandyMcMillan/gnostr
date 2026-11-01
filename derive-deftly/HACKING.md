# **Hacking on derive-deftly (`HACKING.md`)**

Rust procedural macros are a somewhat awkward environment,
and, especially, testing them can be complex.

<!--##toc##-->
   * [Required reading](#required-reading)
   * [User-facing documentation](#user-facing-documentation)
   * [Generated and auto-updated files in the git tree](#generated-and-auto-updated-files-in-the-git-tree)
      * [`tests/pub-export/bizarre-facade/*` etc., updated by `maint/update-bizarre`](#testspub-exportbizarre-facade-etc-updated-by-maintupdate-bizarre)
      * [`Cargo.lock`, updated by `nailing-cargo update`.](#cargolock-updated-by-nailing-cargo-update)
      * [`Cargo.lock.minimal`, updated by `update-minimal-versions`.](#cargolockminimal-updated-by-update-minimal-versions)
      * [Tables of contents in various `*.md`, updated by `maint/update-tocs`.](#tables-of-contents-in-various-md-updated-by-maintupdate-tocs)
      * [Cross-references in `reference.md`, updated by `maint/update-reference-xrefs`](#cross-references-in-referencemd-updated-by-maintupdate-reference-xrefs)
   * [Testing - see `tests/tests.rs`](#testing---see-teststestsrs)
   * [Reporting errors during template parsing and expansion](#reporting-errors-during-template-parsing-and-expansion)
   * [Adding an expansion keyword](#adding-an-expansion-keyword)
      * [Accessing the driver](#accessing-the-driver)
      * [Expansion keywords with content or arguments](#expansion-keywords-with-content-or-arguments)
      * [Adding a keyword that can appear in `${paste }` and/or `${CASE }`](#adding-a-keyword-that-can-appear-in-paste--andor-case-)
      * [Adding a boolean keyword](#adding-a-boolean-keyword)
   * [clippy](#clippy)
      * [Updating the pinned clippy (housekeeping task)](#updating-the-pinned-clippy-housekeeping-task)
      * [clippy `#[allow]`s - strategy and policy](#clippy-allows---strategy-and-policy)
   * [Updating the pinned Nightly Rust (used in tests and CI)](#updating-the-pinned-nightly-rust-used-in-tests-and-ci)
      * [Choosing which Nightly Rust version to update to](#choosing-which-nightly-rust-version-to-update-to)
      * [Updating the nightly version number](#updating-the-nightly-version-number)
      * [Updating the cargo-expand version](#updating-the-cargo-expand-version)
      * [Preparing and merging the changes](#preparing-and-merging-the-changes)
   * [Compatibility testing (and semver updates)](#compatibility-testing-and-semver-updates)

## Required reading

derive-deftly uses types and traits from [`syn`] and [`mod@quote`],
extensively.

It will be very helpful to run one of
```text
maint/build-docs-local --dev
# or
cargo doc --document-private-items --workspace
```
to get a local rendering including for the internal APIs.
That will also get a **rendering of this file with working links**,
as `target/doc/derive_deftly_macros/_doc_hacking/index.html`.

[`NOTES.md`](_doc_notes) has some ideas for the future,
which we may or may not implement.
(Comments welcome!)

## User-facing documentation

Our user-facing documentation is divided between our
`rustdoc` documentation and our `mdbook` source.
The user guide (currently only an introduction)
lives in book/src/*.
See the [`mdbook`](https://rust-lang.github.io/mdBook/)
documentation for more implementation.

To build all the user-facing documentation,
run `maint/build-docs-local` from the top-level directory,
and look in the directory it tells you.

## Generated and auto-updated files in the git tree

The git tree contains some files which are actually
maintained by scripts in `maint/`.

### `tests/pub-export/bizarre-facade/*` etc., updated by `maint/update-bizarre`

"Bizarre" version of `derive-deftly`,
used for [cross-crate compatibility testing](../../pub_b/index.html).

CI will check that these outputs are
up to date with
the normal top-level `Cargo.toml`s, and `pub-b.rs`,
from which they are generated.

### `Cargo.lock`, updated by `nailing-cargo update`.

Example lockfile.

Used in the CI tests,
which (in most tests) pin all of our dependencies.

If you're not a user of
[`nailing-cargo`](https://diziet.dreamwidth.org/tag/nailing-cargo)
you can update this
simply by copying a `Cargo.lock` made with `cargo update`.

### `Cargo.lock.minimal`, updated by `update-minimal-versions`.

Minimal versions of our dependencies,
used for CI testing of our MSRV, etc.

`update-minimal-versions` runs `cargo +nightly update ...`,
so you have to have a Rust Nightly installed.

### Tables of contents in various `*.md`, updated by `maint/update-tocs`.

These are inserted at the `<!--##toc##-->` marker.

Checked by CI, but it's only a warning if it's not up to date.

### Cross-references in `reference.md`, updated by `maint/update-reference-xrefs`

There are `x:...` and `c:...` `<div>`s,
surrounding each heading describing expansions and conditions.

And indexes, at the bottom of the file.

Again, checked by CI, but it's only a warning if it's not up to date.

## Testing - see `tests/tests.rs`

derive-deftly has comprehensive tests.

But, they are complicated
(mostly because testing proc macros is complicated).

You have to use **a particular version of Nightly Rust**.
**See [`tests/tests.rs`](../../derive_deftly_tests/index.html)**
for information on how to run and update the tests.

## Reporting errors during template parsing and expansion

Generally, we use only `syn::Error` as the error type.
Use the [`MakeError`] convenience trait's
[`.error()`](MakeError::error) method
to construct errors.
Often, it is a good idea to generate an error
pointing at the relevant parts of both the driver and the template;
[`MakeError`]'s implementation on
[`[ErrorLoc]`](ErrorLoc) is good for this.

## Adding an expansion keyword

You need to decide if it should be useable in `${paste }`.
Generally, identifiers (or identifier-like things)
strings, 
and types should,
and other things shouldn't.
For now let's assume it shouldn't be useable in `${paste }`.

And, you need to decide if it should be useable
as a boolean expression, in `${if }` etc.
Again, for now, let's assume not.

Add the keyword to [`pub enum SubstDetails`](syntax::SubstDetails)
in `syntax.rs`.
If the keyword isn't a Rust keyword, use its name precisely,
in lowercase.
The enum variannt should contain:
 * Any arguments allowed and supplied, in their parsed form
 * Markers `O::NotInPaste` and `O::NotInBool`,
   as applicable.

Add the keyword to the parser in
`impl ... Parse for Subst`.
Use the provided `keyword!` macro.
For the markers, use `not_in_paste?` and `not_in_bool?`.

The compiler will now insist you add arms to various matches.
Most will be obvious.

The meat of the expansion - what your new keyword means -
is in `SubstDetails::expand`, in `expand.rs`.
For an expansion which isn't permitted in `${paste ..}`,
call 
[`out.append_tokens_with()`](framework::ExpansionOutput::append_tokens_with)
or
[`out.append_tokens()`](framework::ExpansionOutput::append_tokens).

You'll also want to add documentation to `doc/reference.md`,
arrangements for debug printing in `macros/dbg_allkw.rs`,
test cases in `tests/expand/` and maybe `tests/ui/`,
and possibly discussion in `book/src/`.

### Accessing the driver

Information about the driver (and the current variant and field)
is available via [`framework::Context`].

(Use the methods on `Context`, such as
[`field()`](framework::Context::field),
to get access to the per-field and per-variant details,
rather than using `Context.variant`
and open-coding the error handling for `None`.)

### Expansion keywords with content or arguments

Parse the content from `input`, in the `keyword!` invocation.
See `tmeta` et al for an example.

Usually it is best to make a Rust type to represent
the content or arguments,
if there isn't a suitable one already.

To parse a boolean expression, use `Subst<BooleanContext>`.
(Probably, in a `Box`, like in `when`).

Normally it is best to put the `O::Not...` markers
directly in the `SubstDetails` enum variant;
that makes it easier to extract them for use in the `match` arms.

It is fine to have thsee markers in an argument type *as well*.
For a sophisticated example of this, 
see `SubstMeta`,
which allows `... as ...`, except in boolean context.

For named arguments, use [`syntax::ParseUsingSubkeywords`].

### Adding a keyword that can appear in `${paste }` and/or `${CASE }`

Removing `O::NotInPaste` marker from a `SubstDetails` variant
will allow the template to contain that keyword
within `${paste}` and `${CASE}`.

You won't be able to call `out.append_tokens` any more.
Instead, you must use one of the more specific
[`framework::ExpansionOutput`] methods,
such as `append_identfrag` or `append_idpath`.

### Adding a boolean keyword

This is fairly straightforward.
Use `is_enum` (say) as an example.

## clippy

We *do* run clippy, but we turn off all `style` and `complexity` lints.

In CI, we test with a pinned version of clippy, currently 1.79.0,
because clippy often introduces new lints,
and we want to handle that breakage in a controlled fashion.

If your MR branch fails the clippy job, you can repro locally with:

```text
rustup toolchain add 1.79
rustup component add clippy
cargo +1.79 clippy --locked --workspace --all-features
```

### Updating the pinned clippy (housekeeping task)

 * Update the version in `.gitlab-ci.yml`, and above.
 * Run the new clippy and fix or allow lints as appropriate.

### clippy `#[allow]`s - strategy and policy

We put `#![allow(clippy::style, clippy::complexity)]`
in every top-level Rust file.
In tests, we have
`#![allow(clippy::style, clippy::complexity, clippy::perf)]`.

<!-- At some point we might try to use the `Cargo.toml `[lints]` 
     feature, but right now that would be a big MSRV violation -->

(Some files which are sufficiently simple to not trigger any lints,
are lacking these annotations.
We'll add them as needed.)

Feel free to add an `#[allow]` if it seems like
clippy wants you to make the code worse.
We often prefer code which isn't "minimal",
if it seems clearer, or more consistent with other nearby code,
or if it might make future edits easier.

For a clippy false positive, link to the upstream bug report,
eg
`#[allow(clippy::non_minimal_cfg)] // rust-clippy/issues/13007`

## Updating the pinned Nightly Rust (used in tests and CI)

The docker image and the nightly version number `+nightly-YYYY-MM-DD`
must be kept in sync.  `cargo expand` will probably need updating too.

### Choosing which Nightly Rust version to update to

Use this to select a corresponding Nightly Rust and container image:
<br>
<https://www.chiark.greenend.org.uk/~ian/docker-tags-history/>

To parse the json, You can use a rune like this:
<br>
`curl https://www.chiark.greenend.org.uk/~ian/docker-tags-history/rustlang,rust/tags.2024-06-06T13:08+00:00.gz | zcat | jq -r '.results[] | select(.name | match("^nightly-bookworm$")) | .images[] | select(.architecture | match("^amd64"))'`

Pick a date just before upstream Rust branched for a release, since that way
we'll maybe have a less-buggy nightly.  Note the date `YYYY-MM-DD` and
use the `jq` rune in that `README.txt` to get the sha256 image digest.

(The Docker official way seems to be to visit
  <https://hub.docker.com/r/rustlang/rust/tags>
and look for the `nightly-bookworm` tag, or whatever.
However, as far as I can tell historical information is not available,
even though the images *are* retained!)

### Updating the nightly version number

Install the chosen nightly: `rustup toolchain add nightly-YYYY-MM-DD`

Then run:
```text
TRYBUILD=overwrite MACROTEST=overwrite STDERRTEST=overwrite \
cargo +nightly-YYYY-MM-DD test --workspace --all-features
```

**Inspect the output carefully** before committing.
Use `git-grep` on the old nightly date string and fix all the instances.

### Updating the cargo-expand version

Quite likely, you'll need to update cargo-expand too, since it may not
build with the new nightly.

Find the most recent version on `crates.io`,
and `cargo install --locked --version 1.0.NN cargo-expand`

Run the overwriting test rune, above.
**Inspect the output carefully** before committing.
Edit `.gitlab-ci.yml` with search-and-replace to fix all the occurrences.

### Preparing and merging the changes

 1. Make an MR of any preparatory fixes you found you needed to make.
    Such changes ought to work with all compiler versions, and should be
    made into an MR of their own.

 2. When that has merged, make an MR containing *one commit*:

      - gitlab image update
      - `cargo expand` update
      - consequential changes to expected test outputs
      - `Cargo.lock` update (minimal-versions ought not to change here)

    This has to be its own MR because it changes, along the way,
    things that the `every-commit` test assumes don't change.
    Splitting it into multiple MRs arranges to refresh those assumptions.
    <p></p>

 3. Finally, make an MR for any changes you wish to make to this doc.

## Compatibility testing (and semver updates)

New template features can be added,
and that's just a semver addition, as usual;
if a breaking change to a template feature is needed,
that is just a semver breaking change.

More intrusive or technical changes can cause semver breaks
*in crates that export templates*.
See [the public API docs for `define_derive_deftly`](macro@define_derive_deftly#you-must-re-export-derive_deftly-semver-implications)
and [`template_export_semver_check`](macro@template_export_semver_check).
Such breaking changes should be avoided if at all possible.
There are complex arrangements for testing that
compatibility isn't broken accidentally,
mostly in `compat/`.  (See `tests/compat/README.md`.)

If it is necessary to make such a totally breaking change,
consult the git history and see how it was done last time.
(`git annotate` on the implementation of `template_export_semver_check`
or `git log -G` on a relevant version number
may be helpful to find the relevant MR and its commits).
