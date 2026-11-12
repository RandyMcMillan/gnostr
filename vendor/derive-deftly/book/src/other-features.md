<!-- @dd-navbar -->
<!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html">crate top-level</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html#overall-toc">overall toc, macros</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/"><strong>guide/tutorial</strong></a></em> ]</nav>

# Other features

`derive-deftly` has many more features,
that aren't yet explained in this tutorial.
For example:

 * [`fvis`][c:fvis], [`tvis`][c:tvis],
   and [`approx_equal`][c:approx_equal],
   more conditions for dealing with various cases by hand.

 * [`$tdeftype`][x:tdeftype]
   for defining a new data structure
   in terms of features of the input data structure,
   and
   [`$Xattrs`][x:fattrs]
   for passing through attributes.

Full details are in the [reference],
which also has a brief example demonstrating each construct.

<!--
  TODO $tdeftype could be used in difference-solving.md
  but it might be less illustrative if we did.
  -->

<!--

 TODO:

> ## Links
>
>  - Link to more worked and commented examples.

-->

[reference]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html
[c:fvis]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#c:fvis
[c:tvis]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#c:tvis
[c:approx_equal]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#c:approx_equal
[x:tdeftype]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:tdeftype
[x:fattrs]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:fattrs
