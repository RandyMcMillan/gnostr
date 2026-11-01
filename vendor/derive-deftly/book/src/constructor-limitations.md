<!-- @dd-navbar -->
<!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html">crate top-level</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html#overall-toc">overall toc, macros</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/"><strong>guide/tutorial</strong></a></em> ]</nav>

# Limiting a template to one kind of type.

The `Constructor` template above doesn't work for enumerations.
If you try to apply it to one, you'll get
a not-entirely-helpful error message.

In earlier examples,
we've shown how to make templates
that apply to enums as well as structs.
But let's suppose that in this case,
we want our template to be struct-only.

We can tell derive-deftly about this restriction,
to help it generate more useful error messages:

```rust
# use derive_deftly::define_derive_deftly;
define_derive_deftly! {
   Constructor for struct: // (1)

   // The rest is unchanged
   impl<$tgens> $ttype where $twheres {
      pub fn new( $( $fname: $ftype , ) ) -> Self {
          Self {
              $( $fname , )
          }
      }
   }
}
```

(Note the use of
[`for struct`][eo:for]
above at `// (1)`.)

Now if we try to apply our template to an enum,
we'll get a more useful error:


```text,ignore
error: template defined for struct, but applied to enum
```

[eo:for]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#eo:for



