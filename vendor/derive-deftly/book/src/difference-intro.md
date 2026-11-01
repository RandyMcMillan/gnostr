<!-- @dd-navbar -->
<!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html">crate top-level</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html#overall-toc">overall toc, macros</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/"><strong>guide/tutorial</strong></a></em> ]</nav>

# Example 3: Tracking the difference between objects

So far we've seen some of `derive-deftly`'s features
for defining different kinds of templates.

In this section,
we'll extend on what we've learned already
to build a template to track the difference between objects.
Unlike our previous examples,
this template will define a new "difference" type
that mirrors the structure of the type we're applying it to.

Here's our motivating example.
Let's suppose we have some complex structures
and we want to expose the differences between those structures
in a human-readable way.
For example, we might want to write an `assert_eq!` test
that shows only the actual changed fields between two instances,
rather than showing their entire Debug outputs.

> This example is inspired by the [`comparable`] crate,
> which has a bunch of other useful features
> we won't be replicating here.
> If this code seems like something you'd want,
> you should probably use `comparable` instead.

To begin with,
let's assume our crate has defined a `Diff` trait,
looking something like:

```rust
pub trait Diff {
    /// A type that describes the difference between
    /// two values of this type.
    type Difference: std::fmt::Debug;

    fn diff(&self, other: &Self) -> Option<Self::Difference>;
}

macro_rules! assert_no_difference {
    { $a:expr, $b:expr } => {
        match $a.diff($b) {
            Some(diff) => panic!("{} != {}: {:?}",
                stringify!($a),
                stringify!($b),
                diff),
            None => {},
        }
    }
}
```

<!-- TODO a real macro of this kind might want to use
       $crate::Diff::diff(&$a, &$b)
    to avoid trait method dispatch depending on
	what traits are used in the calling scope.
	This is not a derive-deftly-specific point, though.

    In -solving, the example uses Diff:diff rather than $crate::Diff:diff,
    which is almost the same issue.
    -->

We'll also assume that we've implemented our `Diff` trait
on a wide array of types from `std`.

What kind of behavior do we want here?
When we write a template to derive `Diff` on a struct `MyStruct`,
we'll want our template to define a new struct whose fields represent
the changes between the old structure and the new.

```rust
# pub trait Diff { type Difference: std::fmt::Debug; }
# impl Diff for String { type Difference = (); }
# struct SomeOtherStruct;
# impl Diff for SomeOtherStruct { type Difference = (); }
// Example struct
struct MyStruct {
    a: String,
    pub b: SomeOtherStruct
}

// Expected output
#[derive(Debug)]
struct MyStructDiff {
    a: Option< <String as Diff>::Difference >,
    // Let's assume we want the same visibility.
    pub b: Option< <SomeOtherStruct as Diff>::Difference >,
}
```

And when we write a template to derive `Diff` on an enum `MyEnum`,
we'll want our template to define a new enumeration representing the change.

```rust
# pub trait Diff { type Difference: std::fmt::Debug; }
# impl Diff for String { type Difference = (); }
# impl Diff for u32 { type Difference = (); }
# #[derive(Debug,Clone)] struct SomeOtherStruct;
# impl Diff for SomeOtherStruct { type Difference = (); }
// Example enum
#[derive(Clone, Debug)]
enum MyEnum {
    A,
    B(String, String),
    C {
        v1: u32,
        v2: SomeOtherStruct,
    }
}

// Expected output
#[derive(Debug)]
enum MyEnumDiff {
    // We'll never actually generate this variant:
    // if two values are both `MyEnum::A`, they have no difference.
    BothA,

    /// Both items are MyEnum::B, but some value changed.
    BothB(
        Option< <String as Diff>::Difference >,
        Option< <String as Diff>::Difference >,
    ),

    // Both items are  MyEnum::C, but some value changed.
    BothC {
        v1: Option< <u32 as Diff>::Difference >,
        v2: Option< <SomeOtherStruct as Diff>::Difference >,
    },

    // The two items are different variants of MyEnum.
    //
    // (This assumes that MyEnum implements Debug and Clone.)
    VariantChanged {
        original_value: MyEnum,
        new_value: MyEnum,
    }
}
```

Let's try to see how we can make this happen!



[`comparable`]: https://crates.io/crates/comparable


