<!-- @dd-navbar -->
<!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html">crate top-level</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html#overall-toc">overall toc, macros</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/"><strong>guide/tutorial</strong></a></em> ]</nav>

# Making MyClone apply conditionally

Now, for the first time, we will make `MyClone` do something
that Rust's `#[derive(Clone)]` does not:
it will apply only when the fields of a struct are `Clone`.

For example, suppose have a struct like this:
```rust
# use std::sync::Arc;
struct Indirect<T>(Arc<T>, u16);
```
If you try to derive `Clone` on it,
the compiler will generate code something like this:

```rust,ignore
impl<T: Clone> Clone for Indirect<T> { ... }
```

But that `T: Clone` constraint isn't strictly necessary: `Arc<T>` always
implements `Clone`, so your struct could have been be `Clone` unconditionally.

But using derive-deftly,
you can define a template
that derives `Clone` only for the cases
where the _actual_ required constraints are met:

```rust
# use derive_deftly::{define_derive_deftly,Deftly};
define_derive_deftly! {
    MyClone:

    impl<$tgens> Clone for $ttype
    where $twheres
          // This is the new part:
          $( $ftype : Clone , )
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
# use std::{sync::Arc, fmt::Debug} ;
# #[derive(Deftly)]
# #[derive_deftly(MyClone[dbg])]
# struct Example<T> where T: Debug {
#   arc: Arc<T>
# }
# #[derive(Deftly)]
# #[derive_deftly(MyClone[dbg])]
# struct Example2<T> {
#   arc: Arc<T>
# }
```

Here, we are using
[`$ftype`][x:ftype].
("field type") to get the actual type of each field.
Since we're repeating it with `$( ... )`,
we are requiring every field to be `Clone`.

Will this work with non-generic fields,
or if the same field is used more than once?
Once again, yes!
To Rust, this is perfectly valid:

```rust,ignore
impl<T> Clone for Direct
where
    T: Clone,
    T: Clone,
    String: Clone
{
    ...
}
```

## What about that comma?

If you're paying close attention, you might have thought
we had a syntax error above
when we didn't use an explicit comma
after `where $twheres`.

This time,
`derive_deftly` has exactly _one_ piece of cleverness at work.
It makes sure that either `$twheres` is empty,
or that it ends with a comma.
That way, when your template expands
`where $twheres $( $ftype : Clone , )`
it won't produce
where U: Debug + Clone T: Clone`
(which is a syntax error)
or
`where ,`
(which is also a syntax error).

##### A further note on repetition

Note that when we define our additional `where` clauses above, we said
`where $( $ftype: Clone, )`
at the top level.
We didn't have to specify separate of repetition
for variants and fields:
if we have only `$ftype` in a top-level repetition,
`derive_deftly` will iterate over all fields in all variants.

Sometimes, if you do something subtle,
derive-deftly may not be able to figure
out what you're trying to repeat over.
You can use
[`${for fields {...}}`][x:for] or [`${for variants {...}}`][x:for]
to specify explicitly what you want to repeat.
So, above, instead, we could have written
```rust
# use derive_deftly::{Deftly, derive_deftly_adhoc};
# #[derive(Deftly)]
# #[derive_deftly_adhoc]
# enum TestCase<A> { Variant(A) }
# derive_deftly_adhoc! { TestCase:
#     fn testcase<$tgens>() where
${for fields { $ftype: Clone, }}
#     {}
# }
```

You can also use `${for ...}` rather than `$(...)`
in cases where you feel
it makes your macro code clearer.
For example, we could have used `${for}`
to write our `MyClone` example more explicitly like this:

```rust
# use derive_deftly::define_derive_deftly;
define_derive_deftly! {
    MyClone:

    impl<$tgens> Clone for $ttype
    where $twheres
          ${for fields { $ftype: Clone , }}
    {
        fn clone(&self) -> Self {
            match self {
                ${for variants {
                    $vpat => $vtype {
                        ${for fields {
                            $fname: $fpatname.clone(),
                        }}
                    },
                }}
            }
        }
    }
}
```

[t:repetition]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#t:repetition
[x:for]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:for

[x:ftype]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:ftype
