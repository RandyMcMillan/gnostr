<!-- @dd-navbar -->
<!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html">crate top-level</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html#overall-toc">overall toc, macros</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/"><strong>guide/tutorial</strong></a></em> ]</nav>

# Using attributes to make a template take arguments

Let's suppose we want to make our `Constructor` template
a little more flexible:
we'd like to be able to give the `new` function a different name.

The usual way to pass an option to a derive macro
is with an attribute, something like this:
```rust,ignore
#[derive(Deftly)]
#[derive_deftly(Constructor)]
#[deftly(constructor(newfn="function_name"))]
struct Example(...);
```

We can extend our template to take a function name, like this:

```rust
# use derive_deftly::define_derive_deftly;
define_derive_deftly! {
   Constructor for struct:

   impl<$tgens> $ttype where $twheres {
      $tvis fn ${tmeta(constructor(newfn)) as ident} // (1)
      ( $( $fname: $ftype , ) ) -> Self {
          Self {
              $( $fname , )
          }
      }
   }
}

use derive_deftly::Deftly;
#[derive(Deftly)]
#[derive_deftly(Constructor)]
#[deftly(constructor(newfn="construct_example"))]
struct Example {
    a: f64,
    b: String
}
```

Here, instead of specifying "`new`"
for the method name in our template,
we give the name as `${tmeta(constructor(newfn))} as ident`.
This tells the template to look for an
[`#[deftly(constructor(newfn="..."))]`][deftly-attribute]
attribute on the type,
to interpret that attribute as an identifier,
and to use the value of that attribute
in place of the keyword.

The `as ident` portion is mandatory;
without it, derive-deftly can't tell
how to parse the argument to the argument.
Instead of `ident`, you can describe other kinds syntactical types,
such as `str`, `ty`, `path`, or `expr`.
See the [reference][x:tmeta] for a complete list.

<span id="meta-attr-scope">Note that we explicitly
named our attribute as `constructor(newfn)`.
We could instead have specified it as `newfn`,
but that would have the potential to conflict
with attributes interpreted by other templates.
As a best practice, if
we expect our template to be widely used,
we should namespace our attributes as in the example above.
</span>

The
[`$tmeta`][x:tmeta]
("toplevel meta")
keyword that we used here
tells the template
to look at the `#[deftly]` attributes for the _type_.
We can, instead, use
[`$vmeta`][x:vmeta]
("variant meta")
to look for `#[deftly]` attributes for the current _variant_,
or
[`$fmeta`][x:fmeta]
("field meta")
to
to look for `#[deftly]` attributes for the current _field_.

(We'll see [an example of using `$fmeta`](./constructor-advanced-conditionals.md) 
in a little while.)

 <!--
> TODO: Is this the right way to talk about "as lit" and "as ty"?
> I'm thinking not yet.
 -->


[x:tmeta]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:tmeta
[x:vmeta]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:vmeta
[x:fmeta]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:fmeta
[deftly-attribute]: https://docs.rs/derive-deftly/latest/derive_deftly/derive.Deftly.html#deftly-attribute

