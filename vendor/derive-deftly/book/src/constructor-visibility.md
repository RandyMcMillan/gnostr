<!-- @dd-navbar -->
<!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html">crate top-level</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html#overall-toc">overall toc, macros</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/"><strong>guide/tutorial</strong></a></em> ]</nav>

# Working with visibility

Our `Constructor` template above doesn't really make sense
if it's applied to a non-public type;
It would define the `new()` function as public,
when the type is not meant to be exposed outside the crate.
(Rust may even complain that we're declaring
a public function that returns a private type!)

Let's fix this, and have our template give our constructor
the same visibility as the type itself:

```rust
# use derive_deftly::define_derive_deftly;
define_derive_deftly! {
   Constructor for struct:

   impl<$tgens> $ttype where $twheres {
      // (this "$tvis" is new)
      $tvis fn new( $( $fname: $ftype , ) ) -> Self {
          Self {
              $( $fname , )
          }
      }
   }
}
```

Here instead of saying `pub fn new`,
we said `$tvis fn new`.
The
[`$tvis`][x:tvis]
keyword will expand
to the visibility of the top-level type.

There is a similar keyword
[`$fvis`][x:fvis]
that expands to the visibility of the current field.

(Since enum variants are always visible,
there is no such keyword as `$vvis`.
Since enum fields are always visible,
`$fvis` in an enum always expands to `pub`.)

[x:tvis]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:tvis
[x:fvis]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:fvis
