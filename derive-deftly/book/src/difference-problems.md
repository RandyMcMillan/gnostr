<!-- @dd-navbar -->
<!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html">crate top-level</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html#overall-toc">overall toc, macros</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/"><strong>guide/tutorial</strong></a></em> ]</nav>

# Some difficulties when creating our Diff type

In the last section, we saw what we want
our `derive_deftly(Diff)` template to do.
Here we'll analyze a few problems that we need to solve
in order to make it work.

These problems mostly derive from the fact
that we're trying to write a single template
that can either generate a `struct` or an `enum`,
depending on the type we're applying it to,
but they all take different forms.

## Problem 1: What keyword do we use to define? {#kwd}

We want an expansion that gives us `struct` for a struct
and `enum` for an enum.
While we could trivially build one with
`${if is_struct {struct} else {enum}}`,
it seems kind of verbose.

## Problem 2: How do we declare the generics? {#gen-defaults}

When we define our new `FooDiff` struct,
we want it to have the same generics that were used
when we declared `Foo`.

We might think of doing this with something like:
```rust,ignore
struct $<$tname Diff><$tgens> where $twheres { ... }
```

And that might be good enough for many purposes,
but it isn't quite right.
Here's why:
If `Foo` is defined as `struct Foo<L:Display=Bar>`,
then `FooDiff` will only get defined as `struct Foo<L:Display>`.
Thus the corresponding `Diff` type for `Foo`
will not be `FooDiff`, but rather `FooDiff<Bar>`.

We need a way to declare generics _along with their defaults_.

## Problem 3: Extra braces on the enum case. {#enum-braces}

For the enum case, we need our template to generate:
```rust,ignore
enum FooDiff { Variant { ... }, Variant { ... }, ... }
```

But for the struct case, we want:
```rust,ignore
struct FooDiff { ... }
```

Note that there are extra braces in the `enum` case.
How can we generate those?

(We can't use `${if}` to insert a single brace,
since all the arguments to a `derive-deftly` expansion
need to have balanced braces.)

## Problem 4: Defining our structs and variants {#defining}

[Earlier](clone-enums.md)
[we saw](clone-tuple-unit.md)
that when constructure and destructing,
we could pretend that tuple structs and tuple variants
were really `braced` variants with members named `0`, `1`, `2`, and so on;
and that we could pretend that unit structs and unit variants
were just empty `braced` structs.

But this approach doesn't work when _declaring_ a struct or variant:
`struct Foo {}` is not the same as `struct Foo;`,
and `struct Foo { 0: u32 }` is a syntax error!

So we will need a different approach.
We'll need to generate braces and field names
if the struct/variant has named fields;
we'll need to generate parentheses and no field names
if the struct/variant is a tuple;
and we'll need to generate nothing at all
if we have a unit struct/variant.

(Moreover, if we're generating a tuple struct or a unit struct,
we'll need a trailing semicolon,
but if we're generating any kind of a variant,
we'll need a trailing comma.)



## Problem 5: field visibility works differently among structs and enums {#fvis-difference}

We decided above that we wanted each "difference" field
to have the same visibility as the original field in our struct and enum.

[Earlier](constructor-visibility.md),
we saw [`$fvis`][x:fvis] as a way to get the visibility of a field.
But this won't work when we're declaring an `enum`:
since every enum field is public,
`$fvis` for an enum's fields always expands to `pub`,
and Rust won't allow us to write

```rust,ignore
enum FooDiff {
    Variant { pub a: u32; },
}
```


[x:fvis]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:fvis
[x:tgens]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:tgens
