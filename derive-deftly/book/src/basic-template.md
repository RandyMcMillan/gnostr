<!-- @dd-navbar -->
<!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html">crate top-level</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html#overall-toc">overall toc, macros</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/"><strong>guide/tutorial</strong></a></em> ]</nav>

# A first example: derive_deftly(HelloWorld)

Here we'll make an example project
with a simple `derive-deftly` template.
Our template won't do much,
but it will get us started using the system.

<!-- TODO BOOK? The reader likely knows how to make a crate,
     but the text is short and they can skip over
     the parts they already know. -->

To begin with, we'll create a new project:

> ```shell
> $ cargo init --lib dd-hello-world
> $ cd dd-hello-world
> ```

Next, we'll add the `derive-deftly` crate as a new dependency:

```shell
$ cargo add derive-deftly
```

Now we can edit `src/lib.rs` in our project
to define and use a new template.

## Writing our HelloWorld template

There are two parts to using derive-deftly:
specifying _templates_ that you can use to derive new features for your
structs and enums
and then _applying_ those templates to your types.

To define a template, you use
[`define_derive_deftly!`], as in

```rust
use derive_deftly::define_derive_deftly;

define_derive_deftly! {
    HelloWorld:

    impl $ttype {
        pub fn greet() {
            println!("Greetings from {}", stringify!($ttype));
        }
    }
}
```

This is a very simple template: it uses a single expansion: [`$ttype`].
(We'll get into more expansions, and more useful examples, later on.
For now, all you need to know
is that `$ttype` expands to the type
on which you're applying your template.)

Later on, you can apply `HelloWorld` to your own type
with [`#[derive(Deftly)]`[derive-Deftly], as in:

```rust
# use derive_deftly::define_derive_deftly;
# define_derive_deftly! {
#    HelloWorld:
#
#    impl $ttype {
#        pub fn greet() {
#            println!("Greetings from {}", stringify!($ttype));
#        }
#    }
# }
use derive_deftly::Deftly;

#[derive(Clone, Debug, Deftly)]
#[derive_deftly(HelloWorld)]
pub struct MyStruct;

MyStruct::greet();
```

## Limitations of our template

Our template won't work for every type!
For example, suppose that we try to apply it to a generic type:

```rust,ignore
#[derive(Deftly)]
#[derive_deftly(HelloWorld)]
struct Pair<T> {
    first: T,
    second: T,
}
```

In response, derive-deftly will try to expand our template
to something like this:
```rust,ignore
impl Pair {
    pub fn greet() {
        println!("Greetings from {}", stringify!(Pair));
    }
}
```

But that won't work in Rust.
Instead, we would need a template to expand to something more like this:
```rust,ignore
impl<T> Pair<T> {
    ...
}
```

We'll talk about how to make templates work for types like this
in later chapters.


## What's next?

Next, we'll take a detour, and show how to expose our template
so that it can be used in other modules,
and other crates.

After that,
we'll look over a more complex example,
and learn more features of `derive-deftly`.
We'll write a more useful template,
and have it apply to more types.

[`$ttype`]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:ttype
[`define_derive_deftly!`]: https://docs.rs/derive-deftly/latest/derive_deftly//macro.define_derive_deftly.html
[derive-Deftly]: https://docs.rs/derive-deftly/latest/derive_deftly/derive.Deftly.html



