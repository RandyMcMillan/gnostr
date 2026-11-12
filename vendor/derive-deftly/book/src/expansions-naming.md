<!-- @dd-navbar -->
<!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html">crate top-level</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html#overall-toc">overall toc, macros</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/"><strong>guide/tutorial</strong></a></em> ]</nav>

# Naming conventions for derive-deftly expansions

Here we discuss some naming conventions
used in the naming of derive-deftly expansions.

Many derive-deftly expansions'
names start with [a single letter][keyword-initial-letters]
to indicate what information they use:
* `t` for **top-level**
  (the `struct`, `enum`, or `union` you are applying the template to),
* `v` for **variant**
  (a variant of an `enum`),
* or `f` for **field**
  (a single field of a struct or variant).

For example, there is `$ttype` for "top-level type" and `$fname` for
"field name".

(We say "top-level" instead of "struct",
since the "top-level" part of a declaration
can also be an enum or a union.)

Many derive-deftly expansions end with
a short identifier for what they contain.
For example, `$tname` is the name of a top-level type,
`$vname` is the name of a variant,
and `$fname` is the name of a field.
Whenever possible (and logical), we have tried to use the same
identifier for the `t`, `v`, and `f` cases.

[keyword-initial-letters]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#keyword-initial-letters
