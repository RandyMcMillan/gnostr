<!-- @dd-navbar -->
<!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html">crate top-level</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html#overall-toc">overall toc, macros</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/"><strong>guide/tutorial</strong></a></em> ]</nav>

# Making MyClone apply to generics

We've gotten a `MyClone` implementation
that works with several kinds of struct struct, and with enums too.
But here's a structure where our current `MyClone` implementation
will fall flat:

```rust
# use std::fmt::Debug;
struct MyItems<T:Clone, U>
    where U: Clone + Debug
{
    things: Vec<T>,
    items: Vec<U>
}
```

When we go to expand our template, it will generate something like:
```rust,ignore
impl Clone for MyItems { ... }
```

That isn't valid!  For our expansion to compile,
we would need to use the generic parameters
and their "where" constraints, like so:
```rust,ignore
impl<T:Clone, U> Clone for MyItems<T,U>
    where U: Clone+Debug
{ ... }
```

To do this with `derive-deftly`, we have to expand our `MyClone`
templatate as shown below:

Therefore, to get our `MyClone` template working with generic types,
we can expand it to add the same parameters and constraints.

```rust
# use derive_deftly::{define_derive_deftly,Deftly};
define_derive_deftly! {
    MyClone:

    // (The next three lines are new)
    impl<$tgens> Clone for $ttype
    where $twheres
    {
        // (The rest is as before...)
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
# use std::fmt::Debug;
# #[derive(Deftly)]
# #[derive_deftly(MyClone)]
# struct MyItems<T:Clone, U>
#    where U: Clone + Debug
# {
#     things: Vec<T>,
#     items: Vec<U>
# }
#
# #[derive(Deftly)]
# #[derive_deftly(MyClone)]
# enum TestCase<A: Clone> { Variant(A) }
```

Here we meet two new keywords.
[`$tgens`][x:tgens]
("top-level generics") becomes
the generic parameters as declared on the top-level type.
(In our case, that's `T:Clone, U`.)
The [`$twheres`][x:twheres] keyword
("top-level where clauses") becomes
the `where` constraints as declared on the top-level type.
(In our case, that's `U: Clone+Debug`.)

Note that `$ttype` expands to the top-level _type_:
that's now `MyItems<T,U>`,
which is what we want in this case.
If we had wanted only `MyItems` without `<T,U>`,
we would say [`$tname`][x:tname] instead.

<!-- TODO: Forward-reference a discussion of $tdefgens. -->

## But does the syntax work for non-parameterized types?

Will this template still work for non-parameterized types?
Indeed, it will!
To Rust, this syntax is perfectly fine:
```rust
struct Simple {
    a: String
}
// Note empty <> and "where" list:
impl<> Clone for Simple
where
{
    fn clone(&self) -> Self {
        Self {
            a: self.a.clone(),
        }
    }
}
```

[x:tgens]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:tgens
[x:twheres]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:twheres
[x:tname]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:tname
