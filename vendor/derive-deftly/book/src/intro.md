<!-- @dd-navbar book-start -->
<!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html">crate top-level</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html#overall-toc">overall toc, macros</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html">template etc. reference</a> | <strong>guide/tutorial</strong></em> ]</nav>

# Introduction: Using derive-deftly for easy derive macros.

**derive-deftly** is a Rust package
that you can use to define your own `derive` macros
without having to write low-level procedural macros.

The syntax is easy to learn,
but powerful enough to implement macros of
[significant](https://gitlab.torproject.org/nickm/build-deftly)
[complexity](https://gitlab.torproject.org/nickm/serde-deftly).

Just below is a simple example,
to help you get a feel for the system.

### Reference documentation

<!-- This section is aimed primarily at a different audience to the book:
     it's the redirects for readers who want less discursive
     and more formal material.
	 So it is deliberately in a different prose style. -->

There is also comprehensive and rigorous reference material:

* **[Template language reference manual][Reference]**
   (terse, but complete).

* **[rustdoc top-level][rustdoc]**:
  Documents macros exposed by the `derive-deftly` crate.
  Also has links to all the other documentation.

* **[Changelog, MSRV policy and cargo features][Changelog]**.

## Example: deriving field accessor functions

Suppose you want to add accessor functions
for the fields in your `struct`.
You *could* do it by hand, like this:

```rust
struct MyStruct {
   a: Vec<u8>,
   b: (u32, u32),
   c: Option<u16>,
   d: (u32, u32),
   // (more fields here ...)
}
impl MyStruct {
    fn get_a(&self) -> &Vec<u8> {
        &self.a
    }
    fn get_b(&self) -> &(u32, u32) {
        &self.b
    }
    fn get_c(&self) -> &Option<u16> {
        &self.c
    }
    fn get_d(&self) -> &(u32, u32) {
        &self.b
    }
    // (more accessors here...)
}
```

But this is time consuming,
and potentially error-prone.
(Did you see the copy-paste error in `get_d`?)
If you had to define a large number of accessors like this,
or do it for a bunch of types, you might want an easier way.

(Of course, in this case, you could use an
[existing crate](https://crates.io/crates/derive-getters).
But let's suppose that there was no crate meeting your needs,
and you had to build your own.)

Here's how you could define and use
a derive-deftly template to save some time and risk.

```rust
use derive_deftly::{define_derive_deftly, Deftly};

// Defines the template
define_derive_deftly! {
    Accessors:
    // Here, $ttype will expand to the toplevel type; in this case,
    // "MyStruct".
    impl $ttype {
        // This $( ... ) block will be repeated: in this case, once per field.
        $(
            // Here we define a "get_" function: In this definition,
            // $ftype is the type of the field,
            // $fname is the name of the field,
            // and $<...> denotes token pasting.
            fn $<get_ $fname>(&self) -> &$ftype {
                &self.$fname
            }
        )
    }
}

// Applies the template to your struct
#[derive(Deftly)]
#[derive_deftly(Accessors)]
struct MyStruct {
   a: Vec<u8>,
   b: (u32, u32),
   c: Option<u16>,
   d: (u32, u32),
   // (more fields here...)
}
```


> Note 1:
>
> This example is deliberately simplified for readability.
> As written here, it only works for `struct`s
> with no generic parameters.
> Later on, we'll learn how to write templates that work
> to `enum`s, generic types, and more.

> Note 2:
>
> Some of the accessors above aren't the ones you'd write yourself
> in idiomatic Rust:
> You'd probably want to return `&str` instead of `&String`,
> and `&[u8]` instead of `&Vec<u8>`.
>
> Once again, we'll be able to do this more idiomatically
> once we're more familiar with `derive-deftly`.

## What, exactly, can derive-deftly do?

The `derive-deftly` crate helps you do a lot of neat stuff:

* You can define sophisticated _templates_ that apply,
  like `derive()` macros,
  to structs, enums, and unions.
* You can _apply_ your templates to your own structs, enums, and unions.
* You can _export_ your templates for use by other crates.
* Your templates can _define_ new types, functions, methods, and variables:
  any kind of item that Rust allows.
* Within your templates, you can look at _nearly everything_ about
  the input type: fields, variants, attributes, types, and so on.
* Your templates can use _complex control structure_ to
  define configurable behavior or special cases.


Still, there are a number of things that derive-deftly cannot do,
or cannot do yet:

* You can't apply a `derive-deftly` template to anything
  besides a struct, enum, or union.
  For example, you can't apply it to a `fn` declaration
  or an `impl` block.
* Like a `derive` macro,
  a `derive-deftly` template cannot change the type it's applied to:
  You can define a _new_ type,
  but you can't change the definition of the original type.
* The `derive-deftly` template language, though powerful,
  does not attempt to be a general-purpose
  programming language.
  There will always be some problems better solved
  through procedural macros,
  or through changes to the Rust language.
* Because of limitations in the Rust macro system,
  `derive-deftly` templates have to be applied using the
  syntax above.
  (That is, in the example above,
  you need to say `#[derive(Deftly)]` and
  `#[derive_deftly(Accessors)]`.
  You can't define a macro that delivers
  `#[derive(Accessors)]` directly.

## About this book

In the rest of this book,
we'll explain how to use derive-deftly in detail.
We'll try to explain each of its features,
and how to use it to make correct,
reliable templates that handle a broad range of situations.


[Reference]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html
[rustdoc]: https://docs.rs/derive-deftly/latest/derive_deftly/
[Changelog]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_changelog/index.html
