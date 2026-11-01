<!-- @dd-navbar -->
<!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html">crate top-level</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html#overall-toc">overall toc, macros</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/"><strong>guide/tutorial</strong></a></em> ]</nav>

# If you're only deriving once...

If you want, you can
[apply a template to an existing type][adhoc]
without having to name that template.
You might want to do this if you have a template
that you only want to apply to a single struct,
and so you don't want to bother naming it.

Supposing that you wanted to apply the template above to `MyStruct`
and `MyStruct` alone,
you could have said:

```rust
use derive_deftly::{Deftly, derive_deftly_adhoc};

#[derive(Clone, Debug, Deftly)]
#[derive_deftly_adhoc]
pub struct MyStruct;

derive_deftly_adhoc!{
    MyStruct:

    impl $ttype {
        pub fn print_name() {
            println!("The name of this type is {}", stringify!($ttype));
        }
    }
}
```

Of course, that's not so useful yet.
In this case, it would have been easier just to write
`impl MyStruct { pub fn print_name() { ... } }`.
But soon, we'll see how to write more interesting templates,
and how to use them to create much more interesting code.


## What's next

In the next session, we'll start working on a `Clone` example,
and we'll learn how to work with
structs, enums, fields, variants, and more.

(But if you're the kind of person who wants to skip
straight to the reference manual,
you can find it [over here][reference].)


[adhoc]: https://docs.rs/derive-deftly/latest/derive_deftly/macro.derive_deftly_adhoc.html
[reference]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html

