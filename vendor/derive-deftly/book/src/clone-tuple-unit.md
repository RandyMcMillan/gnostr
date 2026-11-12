<!-- @dd-navbar -->
<!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html">crate top-level</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html#overall-toc">overall toc, macros</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/"><strong>guide/tutorial</strong></a></em> ]</nav>

# Will MyClone apply to tuple and unit structs?

Rust defines several kinds of struct:

* structs with fields like `struct Foo {...};`
* tuple structs like `struct Foo(...);`
* and unit structs like `struct Foo;`

So far, you've only applied your `MyClone` template
to a struct with named fields.
But with a tuple struct, or a unit struct,
you might expect it to fail.

Surprisingly, our template still works fine!
This isn't because of any clever trickery
from derive-deftly:
it's just how Rust works.
When you use `MyClone` on tuple or unit struct,
it will expand to code like this the example below.
And this syntax,
though not the normal way to work with these types,
is still valid Rust!

```rust
struct TupleStruct(String, Option<String>);
impl Clone for TupleStruct {
    fn clone(&self) -> Self {
        Self {
            0: self.0.clone(),
            1: self.1.clone(),
        }
    }
}

struct UnitStruct;
impl Clone for UnitStruct {
    fn clone(&self) -> Self {
        Self {
        }
    }
}
```

This will be a common theme when using derive-deftly:
Rust often lets you use a slightly unidiomatic syntax,
so that you can handle many different cases
in the same way.
