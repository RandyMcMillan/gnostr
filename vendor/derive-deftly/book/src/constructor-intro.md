<!-- @dd-navbar -->
<!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html">crate top-level</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html#overall-toc">overall toc, macros</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/"><strong>guide/tutorial</strong></a></em> ]</nav>

# Example 2: Defining a constructor function

In this section,
we'll be using another example to demonstrate
more of what `derive-deftly` can do.

We'll be building a `Constructor` template
to define a `new()` function for a struct,
without having to write out all of its arguments.

Let's start with the following (struct-only) template:

```rust
# use derive_deftly::define_derive_deftly;
define_derive_deftly! {
   Constructor:

   impl<$tgens> $ttype where $twheres {
      pub fn new( $( $fpatname: $ftype , ) ) -> Self {
          Self {
              $( $fname: $fpatname , )
          }
      }
   }
}
```

When you apply the above template to a type like this:

```rust
# use derive_deftly::define_derive_deftly;
# define_derive_deftly! {
#   Constructor:
#
#   impl<$tgens> $ttype where $twheres {
#       pub fn new( $( $fpatname: $ftype , ) ) -> Self {
#           Self {
#               $( $fname: $fpatname , )
#           }
#       }
#   }
# }
use derive_deftly::Deftly;
#[derive(Deftly)]
#[derive_deftly(Constructor)]
struct Ex<A> {
  a: f64,
  b: A,
  c: String
}
```

You'll get a constructor like this:
```rust
# struct Ex<A> { a: f64, b: A, c: String }
impl<A> Ex<A> {
    pub fn new( f_a: f64, f_b: A, f_c: String ) -> Self {
        Self { a: f_a, b: f_b, c: f_c }
    }
}
```

So far, there aren't any new techniques at work here.
We'll add some more down below.

Note the use of `$fpatname` for the function arguments, rather than `$fname`.
This makes the macro work with tuple structs:
a tuple struct has field "names" like `0` and `1`,
which aren't useable as variable names.
