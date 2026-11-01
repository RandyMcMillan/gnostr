<!-- @dd-navbar -->
<!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html">crate top-level</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html#overall-toc">overall toc, macros</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/"><strong>guide/tutorial</strong></a></em> ]</nav>

# Making MyClone apply to enumerations

At this point, you've probably noticed
that we've defined `MyClone` to apply to `struct`s only,
but it won't (yet) work on `enum`s.
Let's fix that!

Suppose that we have enumeration defined like this:

```rust
enum AllTypes {
    NoData,
    Tuple(u8, u16),
    Struct { a: String, b: String }
}
```
We want to make sure that
MyClone can recognize and re-construct
each of the three variants.

We can do that as follows:

```rust
# use derive_deftly::define_derive_deftly;
define_derive_deftly! {
    MyClone:

    impl Clone for $ttype
    {
        fn clone(&self) -> Self {
            match self {
                $(
                    $vpat => $vtype {
                        $(
                            $fname: $fpatname.clone(),
                        )
                    },
                )
            }
        }
    }
}
```

Note that now we have two levels of nested repetition.
First, we match once for each variant.
(This is at the `$vpat` and `$vtype` level.)
Then we match once for each field of each variant.
(This is at the `$fname` and `$fpatname` level.)

Let's go over the new expansions here.
First, we have
[`$vpat`][x:vpat] ("variant pattern"):
that expands to a pattern that can match and deconstruct
a single variant.
Then, we have
[`$vtype`][x:vtype] ("variant type"):
that's the type of the variant,
suitable for use as a constructor.
Then, inside the variant, we have `$fname`:
that's our field name, which we've seen it before.
Finally, we have
[`$fpatname`][x:fpatname] ("field pattern name"):
that is the name of the variable that we used for this field
in the pattern that deconstructed it.

When we apply `MyClone` to our enumeration,
we get something like this:
```rust
# enum AllTypes { NoData, Tuple(u8, u16), Struct { a: String, b: String } }
impl Clone for AllTypes {
    fn clone(&self) -> Self {
        match self {
            AllTypes::NoData {} => AllTypes::NoData {},
            AllTypes::Tuple {
                0: f_0,
                1: f_1,
            } => AllTypes::Tuple {
                0: f_0.clone(),
                1: f_1.clone()
            },
            AllTypes::Struct {
                a: f_a,
                b: f_b,
            } => AllTypes::Struct {
                a: f_a.clone(),
                b: f_b.clone()
            },
        }
    }
}
```

Note the `f_` prefixes on the variables in our patterns,
and in our `$fpatname` expansions:
without them, our tuple variants would contain invalid patterns
like `0: 0`.

Note also that our template above will still work fine on a regular struct,
even though it's written for an `enum`.
If we apply the version of `MyClone` above
to `struct Example { a: u8, b: String }`,
we get this:

```rust
# struct Example { a: u8, b: String }
impl Clone for Example {
    fn clone(&self) -> Self {
        match self {
            Example {
                a: f_a,
                b: f_b,
            } => Example {
                a: f_a.clone(),
                b: f_b.clone(),
            }
        }
    }
}
```

So (in this case at least)
we were able to write a single template expansion
that worked for both `struct`s and enum`s.

[x:vpat]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:vpat
[x:vtype]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:vtype
[x:fpatname]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:fpatname
