<!-- @dd-navbar -->
<!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html">crate top-level</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html#overall-toc">overall toc, macros</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/"><strong>guide/tutorial</strong></a></em> ]</nav>

# Making Constructor set fields with Default

Sometimes, we'd like to make a template template
that behave in different ways for different fields.
For example, let's suppose that we want our `Constructor` template
to be able to set fields to their default values,
and not take them as arguments.

We can do this with an explicit conditional for each field:
```rust
# use derive_deftly::define_derive_deftly;
define_derive_deftly! {
   Constructor:

   impl<$tgens> $ttype where $twheres {
     $tvis fn
        // This is the function name, same as before.
        ${if tmeta(constructor(newfn)) {
            ${tmeta(constructor(newfn)) as ident}
          } else {
            new
          }
        }
     (
       // These are the function arguments:
       $(
          ${when not(fmeta(constructor(default))) } // (1)
          $fpatname: $ftype ,
        )
     ) -> Self {
          Self {
              $( $fname:
                  ${if fmeta(constructor(default)) {
                    ::std::default::Default::default() // (2)
                  } else { $fpatname } }
                 , )
          }
      }
   }
}

use derive_deftly::Deftly;
#[derive(Deftly)]
#[derive_deftly(Constructor)]
struct Foo {
    #[deftly(constructor(default))]
    s: Vec<String>,
    n: u32,
}
```

Here we're using a new construct:
[`$when`][x:when].
It's only valid inside a loop like `$( ... )` or `${for ...}`.
It causes the expansion of the loop body to be suppressed
whenever the condition is not true.

The condition in this cases is `not(fmeta(constructor(default)))`.
(See `// (1)`.)
You've seen `fmeta` before;
it's true when a given attribute is present on the current field.
The [`not`][c:not] condition
is just how we express negation.
All together, this `$when` keyword causes each field
that has `#[deftly(Constructor(default))]` applied to it
to be omitted from the list of arguments
to the `new()` function.

> Note at `// (2)` that we're using `::std::default::Default::default()`,
> rather than calling `Default::default()` unconditionally.
> Remember, macros expand at the position where they are invoked,
> and it's possible that we'll be invoked in a module
> that has been built [without the standard prelude][no_implicit_prelude].

<!-- TODO: Write a further section about hardening macros against weird
      environments, then link to it here. -->

Besides `$not`,
you can use other boolean operators in conditions too:
there is an
[`any(...)`][c:any]
that is true
whenever at least one of its arguments is true,
and an
[`all(...)`][c:all]
that is true
when _all_ of its arguments are true.

[c:all]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#c:all
[c:any]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#c:any
[c:not]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#c:not
[x:when]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:when
[no_implicit_prelude]: https://doc.rust-lang.org/reference/names/preludes.html#the-no_implicit_prelude-attribute
