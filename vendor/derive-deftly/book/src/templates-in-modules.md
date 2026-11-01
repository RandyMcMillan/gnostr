<!-- @dd-navbar -->
<!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html">crate top-level</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html#overall-toc">overall toc, macros</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/"><strong>guide/tutorial</strong></a></em> ]</nav>

# Templates inside modules

When you define a template, derive-deftly turns it into a definition
of a `macro_rules!` macro.
In the example above, that would be `derive_deftly_template_HelloWorld`.

Unfortunately,
[Rust's rules for scoping][macro-scoping] of `macro_rules!` macros
are awkward and confusing.
In general, there are two ways to expose a macro;
we'll go over how to use each one
for a derive-deftly template.

## With path-scoped macros { #path-scope }

With modern versions of Rust
(Rust 2018 and later),
you can use
[path-based scope] to define the scope of a macro.
With this method, you use
`pub use`
(or `pub(crate) use`, etc)
to define the scope for a macro,
and allow it to be accessed by its path.

```rust
pub mod hello_world {
    use derive_deftly::define_derive_deftly;
    define_derive_deftly! {
        HelloWorld: /* ... */
    }
    pub(crate) use derive_deftly_template_HelloWorld;
}

mod caller {
    use derive_deftly::Deftly;
    use super::hello_world::derive_deftly_template_HelloWorld;

    #[derive(Deftly)]
    #[derive_deftly(HelloWorld)]
    pub struct MyStruct;

    // Alternatively, you use the path, and avoid having to say "use":
    #[derive(Deftly)]
    #[derive_deftly(super::hello_world::HelloWorld)]
    pub struct MyOtherStruct;
}
# fn main() {} // so that the above is not wrapped in fn main().
```

This is generally the better method to use.


## Using `#[macro_use]`

We can also expose a macro using [textual scope] and [`macro_use`]:
We prefix
that module's `mod` statement with `#[macro_use]`.
If you do this, the module defining the macro
must come before the module where you use the macro.
(Also, the macro's name is never scoped within the module.)

```rust
#[macro_use] // make HelloWorld visible outside the module (within the crate)
mod hello_world {
    use derive_deftly::define_derive_deftly;
    define_derive_deftly! {
        HelloWorld: /* ... */
    }
}
mod caller { // must come after mod hello_world
    use derive_deftly::Deftly;
    #[derive(Deftly)]
    #[derive_deftly(HelloWorld)] // not hello_world::HelloWorld
    pub struct MyStruct;
}
```


[macro-scoping]: https://doc.rust-lang.org/reference/macros-by-example.html#scoping-exporting-and-importing
[textual scope]: https://doc.rust-lang.org/reference/macros-by-example.html#textual-scope
[`macro_use`]: https://doc.rust-lang.org/reference/macros-by-example.html#the-macro_use-attribute
[path-based scope]: https://doc.rust-lang.org/reference/macros-by-example.html#path-based-scope
