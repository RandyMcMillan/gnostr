# derive-deftly: An ergonomic replacement for many proc macros

<!-- @dd-navbar toplevel https://docs.rs/derive-deftly/@version@/derive_deftly -->
<!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <strong>crate top-level</strong> | <a href="https://docs.rs/derive-deftly/0.14.6/derive_deftly/index.html#overall-toc">overall toc, macros</a> | <a href="https://docs.rs/derive-deftly/0.14.6/derive_deftly/doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/">guide/tutorial</a></em> ]</nav>

`derive-deftly` allows you to write macros which are driven
by Rust data structures, just like proc macro derive macros,
but without having to wrestle with the proc macro system.

**Stability**

We intend to make a 1.x release of this crate very soon.
We expect that derive-deftly 1.x will be compatible with 0.14.x.

## Overview

You can write an ad-hoc template,
which can speak about the fields and types in the data structure.
You can also define named templates and apply them to multiple structures:
effectively, you can define your own derive macro.

You **don't** need to make a separate proc macro crate,
write to the `syn` and `proc_macro` APIs.
take care to properly propagate compile errors,
or, generally, do any of the things that
make writing proc macros so complicated.

The template language resembles the "expander" part
of a `macro_rules` macro,
but you don't have to write the "matcher" part:
derive-deftly parses the input data structure for you,
and makes the pieces available via predefined expansion variables.

Further documentation is available in the `doc_` module(s)
and the docs for the individual proc macros.

## Simple example - providing `Vec` containing enum variant names

```
use derive_deftly::{define_derive_deftly, Deftly};

define_derive_deftly! {
    ListVariants:

    impl $ttype {
        fn list_variants() -> Vec<&'static str> {
            vec![ $( stringify!( $vname ) , ) ]
        }
    }
}

#[derive(Deftly)]
#[derive_deftly(ListVariants)]
enum Enum {
    UnitVariant,
    StructVariant { a: u8, b: u16 },
    TupleVariant(u8, u16),
}

assert_eq!(
    Enum::list_variants(),
    ["UnitVariant", "StructVariant", "TupleVariant"],
);
```

## Next steps

Why not have a look at our friendly [user guide]?
It will walk you through derive-deftly's most important features,
with a number of worked examples.

Alternatively, there is comprehensive
[reference documentation][navbar-versioned-url-overall-toc].

[user guide]: https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/
<!-- next line is automatically maintained by update-docs-navbars -->
[navbar-versioned-url-overall-toc]: https://docs.rs/derive-deftly/0.14.6/derive_deftly/index.html#overall-toc
