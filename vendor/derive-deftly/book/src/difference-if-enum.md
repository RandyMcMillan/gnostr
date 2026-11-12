<!-- @dd-navbar -->
<!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html">crate top-level</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html#overall-toc">overall toc, macros</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/"><strong>guide/tutorial</strong></a></em> ]</nav>

# A brute-force approach for applying to structs and enums

One way we can solve [our problems](./difference-problems.md)
with writing a `derive_deftly(Diff)` template
is a brute-force approach.
Here we just use [conditionals](constructor-conditionals.md)
to write different versions of our template
for the `struct` case and the `enum` case.
To handle the difference between tuple and unit structs and variants,
we can use conditionals as well.

Here are the new conditions we'll need:
  * [`is_enum`][c:is_enum] - True if the top-level type is an enum.
  * [`is_struct`][c:is_struct] - True if the top-level type is a struct.
  * [`v_is_unit`][c:v_is_unit] - True if the current variant
     (or the top-level type, in the case of a struct)
     is a unit struct/variant.
  * [`v_is_tuple`][c:v_is_tuple] - True if the current variant
     (or the top-level type, in the case of a struct)
     is a tuple struct/variant.
  * [`v_is_named`][c:v_is_named] - True if the current variant
     (or the top-level type, in the case of a struct)
     is a struct/variant with named fields.

With these conditions, we can write our template,
albeit with quite a lot of redundancy.

> Note 1: You may want to refresh your memory about
> [`${select1}`](constructor-conditionals.md#select1)
> and [`$<pasting>`](paste-and-case.md).
>
> Note 2: We're ignoring generics in this example, for clarity.

<!-- TODO: When I use $crate:: below, the rustdoc doesn't pass its tests. -->
<!-- We should investigate. -->

```rust
# pub trait Diff { type Difference: std::fmt::Debug; }
# impl Diff for u32 { type Difference = (); }
# use derive_deftly::{define_derive_deftly, Deftly};
define_derive_deftly!{
    Diff:

    ${define DIFF_TYPE
        { Option< <$ftype as Diff>:: Difference> }
    }

    ${select1 is_struct {
        ${select1 v_is_unit {
            #[derive(Debug)]
            $tvis struct $<$tname Diff>;
        } else if v_is_tuple {
            #[derive(Debug)]
            $tvis struct $<$tname Diff>(
                ${for fields {$DIFF_TYPE , }}
            );
        } else if v_is_named {
            #[derive(Debug)]
            $tvis struct $<$tname Diff> {
                $(
                    $fvis $fname: $DIFF_TYPE,
                )
            }
        }} // end select1
    } else if is_enum {
        #[derive(Debug)]
        $tvis enum $<$tname Diff> {
            $(
                ${select1 v_is_unit {
                    // nothing to do; we don't emit anything here.
                } else if v_is_tuple {
                    $<Both $vname>(
                        ${for fields {$DIFF_TYPE , }}
                    ),
                } else if v_is_named {
                    $<Both $vname> {
                        $(
                            $fname: $DIFF_TYPE,
                        )
                    },
                }} // end select1
            ) // end iteration over variants

            VariantChanged {
                original_value: $ttype,
                new_value: $ttype,
            },
        }
    }} // end select1
    impl Diff for $tname {
        type Difference = $<$tname Diff>;

        // ... and at this point, you still have to define
        // an implementation for `fn diff()`: good luck!
    }
}
# use derive_deftly_template_Diff;
# #[derive(Clone,Debug,Deftly)]
# #[derive_deftly(Diff)]
# struct Unit;
# #[derive(Clone,Debug,Deftly)]
# #[derive_deftly(Diff)]
# struct Tuple(u32, u32);
# #[derive(Clone,Debug,Deftly)]
# #[derive_deftly(Diff)]
# struct Named { a: u32, b: u32 }
# #[derive(Clone,Debug,Deftly)]
# #[derive_deftly(Diff)]
# enum Enum { Un, Tup(u32,u32), Nam { a: u32, b: u32 } }
```

This is a viable approach, but not a very maintanable one:
look how we've had to copy out our definition separately
for every possible Rust syntax!
We managed to save some repetition with `$define`,
but we can still do better.

<!-- TODO: I'd rather not have to use $for above; see #110. -->


[c:is_enum]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#c:is_enum
[c:is_struct]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#c:is_struct
[c:v_is_unit]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#c:v_is_unit
[c:v_is_tuple]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#c:v_is_tuple
[c:v_is_named]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#c:v_is_named
