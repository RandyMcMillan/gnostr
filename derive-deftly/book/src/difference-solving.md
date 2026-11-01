<!-- @dd-navbar -->
<!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html">crate top-level</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html#overall-toc">overall toc, macros</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/"><strong>guide/tutorial</strong></a></em> ]</nav>

# A better solution to our problems with Diff

In previous sections, we've seen
[a few problems](difference-problems.md)
with writing a template that can define either a struct or an enum.
We built a [brute-force solution](difference-if-enum.md)
that was workable if not especially elegant.
Here, we'll show a better way.

We'll start by revisiting those problems;
this time we'll show a better solution to each one.
After we've gone over those solutions,
we'll put them together into a working template.

## Revisiting our problems

So, how can we make our template?

### Solving problem 1: `$tdefkwd` for a top-level keyword

We needed [a way to get the right keyword](difference-problems.md#kwd)
for our new type.
This is available nicely as [`$tdefkwd`][x:tdefkwd]
("toplevel definition keyword"),
which expands to `struct`, `union`, or `enum` as appropriate.


### Solving problem 2: `$tdefgens` for generics with defaults

We needed a way to
[declare generics along with their defaults](difference-problems.md#gen-defaults).
This come in the form of [`$tdefgens`][x:tdefgens],
which gives the generics from the top-level type
along with their bounds
_and_ their defaults.

### Solving problem 3: `$tdefvariants` to brace our variants as needed

We needed some way to
[wrap our enum variants in braces](difference-problems.md#enum-braces),
but leave our structs unwrapped.

derive-deftly solves this problem with
the [`$tdefvariants`][x:tdefvariants] expansion,
which adds braces only when the top-level type is an enum.
In other words, `${tdefvariants ARG}` expands to `{ARG}` for an enum,
but `ARG` otherwise.

### Solving problem 4: `$vdefbody` and `$fdefine` for struct/variant types

We needed some way to make
[appropriate type and field declarations](difference-problems.md#defining)
no matter whether the current struct or variant
is a unit struct/variant, a tuple struct/variant,
or a struct with named fields/fields.

We solve this problem in two parts.

First, the [`$vdefbody`][x:vdefbody] expansion gives us the
appropriate syntax to declare a variant or structure
corresponding to the current variant/structure,
with the necessary braces, parentheses, and/or trailing semicolon or comma.

Specifically, `${vdefbody VNAME FIELDS}` will expand to one of:

 * `FIELDS;` for unit structs (though typically FIELDS will be empty)
 * `(FIELDS);` for tuple structs
 * `{FIELDS}` for structs with named fields
 * `VNAME FIELDS,` for a unit variant in an enum
   (though typically FIELDS will be empty)
 * `VNAME(FIELDS),` for a tuple variant in an enum
 * `VNAME { FIELDS },` for a variant with named fields in an enum.

Second,
to provide a field name and colon if appropriate, we use
[`${fdefine NAME}`][x:fdefine],
which expands to `NAME:` if fields are named,
and to nothing otherwise.

### Solving problem 5: `$fdefvis` for visibility

We needed some way to define
[appropriate visibility](difference-problems.md#fvis-difference)
for struct fields,
given that `pub` isn't allowed to appear in an enum definition.

For this, we use [`$fdefvis`][x:fdefvis],
which expands to the visibility of the current field in a struct,
and to nothing in an enum.


## Putting it all together

This probably sounds like a lot,
and admittedly it can be tricky to synthesize.

As a template, it's handy to start by modifying this syntax,
which just reproduces the original type with a new name.
```rust,ignore
$tvis $tdefkwd $<$tname Copy><$tdefgens>
${tdefvariants $(
    ${vdefbody $vname $(
        $fdefvis ${fdefine $fname} $ftype,
    ) }
) }
```

Based on that, we produce the following.
(As an added bonus, we'll define the actual `diff` method!)

<!-- TODO: When I use $crate:: below, the rustdoc doesn't pass its tests. -->
<!-- We should investigate. -->

```rust
# pub trait Diff { type Difference: std::fmt::Debug; fn diff(&self, o:&Self) -> Option<Self::Difference>; }
# impl Diff for u32 { type Difference = (); fn diff(&self,o:&Self) -> Option<()> {None} }
# use derive_deftly::{define_derive_deftly, Deftly};
define_derive_deftly!{
    Diff:

    ${if is_union {${error "Unions aren't supported"}}}

    ${define DIFF_TYPE
        { Option< <$ftype as Diff>:: Difference> }
    }

    #[derive(Debug)]
    $tvis $tdefkwd $<$tname Diff><$tdefgens>
    ${tdefvariants
        $(
            ${vdefbody $<Both $vname>
                $(
                    $fdefvis ${fdefine $fname} $DIFF_TYPE ,
                )
            }
        )
        ${if is_enum {
            VariantChanged {
                original_value: $ttype,
                new_value: $ttype,
            },
        }}
    }

    impl<$tgens> $<$ttype Diff> where $twheres {
        /// Helper: Return this value if it reflects a real change.
        fn if_is_a_change(self) -> Option<Self> {
            match &self {
                $(
                    // (1)
                    ${vpat self=$<$ttype Diff> vname=$<Both $vname>} => {
                        if $( $fpatname.is_none() && ) true  {
                            return None;
                        }
                    }
                )
                ${if is_enum { Self::VariantChanged{..} => {} }}
            }
            return Some(self);
        }
    }

    impl<$tgens> Diff for $ttype where
        ${if is_enum {$ttype: Clone,}}
        $twheres
    {
        type Difference = $<$ttype Diff>;
        fn diff(&self, other: &Self) -> Option<Self::Difference> {
            match (self, other) {
                $(
                    // (2)
                    ($vpat, ${vpat fprefix=other_}) => {
                        // (3)
                        ${vtype self=$<$ttype Diff> vname=$<Both $vname>} {
                            $(
                                $fname : Diff::diff($fpatname, $<other_ $fname>),
                            )
                        }
                    }
                ) // End variants.
                ${if is_enum {
                    (original_value, new_value) => {
                        Self::Difference::VariantChanged {
                            original_value: original_value.clone(),
                            new_value: new_value.clone(),
                        }
                    }
                }}
            }.if_is_a_change()
        }
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

With understanding,
this approach should appear much terser and more logical
than the many-cases approach from the
[last section](difference-if-enum.md).

We've introduced a few previously unseen expansions:
let's talk about them now.

At the beginning, we use [`$error`][x:error].
If it is ever expanded,
then the whole template expansion is rejected with a compile-time error.
We use it to make sure that nobody tries to apply our template
to a `union`.

We use [`$vpat`][x:vpat] in a new form.
As discussed [in the reference][x:vpat],
`$vpat` can take arguments to change its behavior.
We use this feature twice:

- At `// (1)`
  we use the `self` argument
  to make our pattern destructure the Diff object
  rather than the original top-level type,
  and we use `vname` to destructure a `$<Both $vname>` variant
  rather than the original variant name.
- At `// (2)`,
  we use the `fprefix` argument to give the second instance of our pattern
  a different set of binding names,
  so they will not conflict with the first instance.

Finally, we pass arguments to [`$vtype`][x:vtype] at `// (3)`,
so that instead of constructing an instance of the current variant,
it constructs the `*Diff` type with an appropriate `Both*` name.


[x:tdefkwd]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:tdefkwd
[x:tdefvariants]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:tdefvariants
[x:vdefbody]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:vdefbody
[x:fdefine]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:fdefine
[x:fdefvis]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:fdefvis
[x:tdefgens]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:tdefgens
[x:error]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:error
[x:vpat]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:vpat
[x:vtype]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:vtype
