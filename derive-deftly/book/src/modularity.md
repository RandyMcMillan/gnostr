<!-- @dd-navbar -->
<!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html">crate top-level</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html#overall-toc">overall toc, macros</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/"><strong>guide/tutorial</strong></a></em> ]</nav>

# Using templates from other crates and modules

In the previous section,
we learned how to crate our first template.
We used `derive-deftly` to define a template called `HelloWorld`
that could print the name of a type.

Before we go any further,
we'll discuss how to use that template
from other modules within the same crate,
and how to expose that template so that other crates can use it.

We'll also discuss a [convenient way]
to define a template that only needs to be used once.

[convenient way]: ./adhoc.md
