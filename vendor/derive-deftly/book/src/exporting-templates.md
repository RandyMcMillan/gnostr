<!-- @dd-navbar -->
<!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html">crate top-level</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html#overall-toc">overall toc, macros</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/"><strong>guide/tutorial</strong></a></em> ]</nav>

# Exporting templates

But now suppose that you want to expose your template,
so that people can use it from any crate.

To do this, you use
[`export` before the name of your macro][exporting-a-template]:
```rust
pub trait HelloWorld {
    fn greet();
}

derive_deftly::define_derive_deftly! {
    /// Derives `HelloWorld`, providing `greet`
    export HelloWorld:
    impl $crate::HelloWorld for $ttype {
        fn greet() {
            println!("Greetings from {}", stringify!($ttype));
        }
    }
}
# fn main() {}
```

> Declaring our template as `export HelloWorld` in this way
> is equivalent to marking `derive_deftly_template_HelloWorld`
> with `#[macro_export]`.
> Doing this puts the template into the root of the crate.
> You can make the template visible elsewhere using `pub use`.

Note that this time,
we've defined `HelloWorld` as a trait,
and we've changed our template to refer to that trait as
`$crate::HelloWorld`.
The
[`$crate`][x:crate]
syntax will expand to the name of the crate
in which our template was defined,
so that when later we expand this template in a _different_ crate,
it will find the right trait.
(Otherwise, it would fail if the `HelloWorld` trait were not in scope.)

We've also added a
[doc comment],
which will appear in the public API documentation for our crate.

Additionally,
[we need to re-export `derive_deftly`][must-re-export]
from our crate, so that when our template is applied to user types,
a compatible derive-deftly engine is used.
And we should also invoke
[`derive_deftly::template_export_semver_check`]
once,
somewhere in our crate:
this will tells us about any implications for your crate's semver,
when you change your `Cargo.toml` to upgrade the `derive-deftly` crate.

```rust,ignore
// At the root of our crate:

#[doc(hidden)]
pub use derive_deftly;

// Use the current version of derive_deftly here:
derive_deftly::template_export_semver_check!("0.11.0");
```

Now, when somebody wants to use our template from a different crate,
they can do it like this:

```rust,ignore
// Let's pretend our crate is called hello_world.
use hello_world::{
    // This is the trait we defined...
    HelloWorld,
    // This is the macro that makes our template work.
    // (We might come up with a better syntax for this later).
    derive_deftly_template_HelloWorld
};
use derive_deftly::Deftly;

#[derive(Deftly)]
#[derive_deftly(HelloWorld)]
struct TheirStructure {
    // ...
}
```

Note that exporting a template to other crates with `export`
doesn't affect its visibility *within your crate*.
You may still need `#[macro_use]`.

## What's next

Now that we've explained how to expose a template,
it's time to learn about a simpler syntax,
for template we only want to use once.



[`derive_deftly::template_export_semver_check`]: https://docs.rs/derive-deftly/latest/derive_deftly/macro.template_export_semver_check.html
[must-re-export]: https://docs.rs/derive-deftly/latest/derive_deftly//macro.define_derive_deftly.html#you-must-re-export-derive_deftly-semver-implications
[doc comment]: https://docs.rs/derive-deftly/latest/derive_deftly//macro.define_derive_deftly.html#docs-in-define
[exporting-a-template]: https://docs.rs/derive-deftly/latest/derive_deftly//macro.define_derive_deftly.html#exporting-a-template-for-use-by-other-crates
[x:crate]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:crate
