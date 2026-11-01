<!-- @dd-navbar -->
<!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html">crate top-level</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html#overall-toc">overall toc, macros</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/"><strong>guide/tutorial</strong></a></em> ]</nav>

# Debugging templates

When writing complex templates,
it can sometimes be hard to figure out compiler errors.
`derive-deftly` has some features which can help:

#### Syntax checking the expansion

You can use the [`expect`][eo:expect] option
to tell derive-deftly what your macro is supposed to produce:

```rust
# use derive_deftly::{Deftly, define_derive_deftly};
define_derive_deftly! {
   Constructor for struct, expect items:

   impl<$tgens> $ttype where $twheres {
      // ...
   }
}
# #[derive(Deftly)]
# #[derive_deftly(Constructor)]
# struct Test;
```

If the expansion fails to syntax check,
you'll get not only an error pointing at
the part of the template or structure which seems wrong,
but also error messages pointing into a *copy of the actual expansion*.
Hopefully you'll be able to see what's wrong there.

If your macro is supposed to expand to an expression,
you can write `expect expr` instead of `expect items`.

Alternatively,
you can request this syntax check
[when you invoke the macro][expansion-options]:

```rust
# use derive_deftly::{Deftly, define_derive_deftly};
# define_derive_deftly! { Constructor: }
#[derive(Deftly)]
#[derive_deftly(Constructor[expect items])]
struct MyStruct { /* ... */ }
```

> Note: The above section isn't really logical: "expect items" is always
> the case for templates defined with `define_derive_deftly`:
> only for adhoc templates is `expect expr` reasonable.
>
> TODO: Rewrite and move this section.

#### Seeing a copy of the expansion

Sometimes, the expansion is syntactically valid,
but is wrong in some other way.

You can use the [`dbg`][eo:dbg] option to tell derive-deftly to
print a copy of the expansion.

```rust
# use derive_deftly::{Deftly, define_derive_deftly};
# define_derive_deftly! { Constructor: }
#[derive(Deftly)]
#[derive_deftly(Constructor[dbg])]
struct MyStruct { /* ... */ }
```

You'll see the compiler print something like this:

```ignore
---------- derive-deftly expansion of Constructor for MyStruct (start) ----------
impl<> MyStruct where {
    pub fn new(...) { ... }
}
---------- derive-deftly expansion of Constructor for MyStruct (end) ----------
```

Just as with the `dbg!` macro, you don't want to
leave this in your production code.

[eo:expect]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#eo:expect
[expansion-options]: https://docs.rs/derive-deftly/latest/derive_deftly/derive.Deftly.html#expansion-options
[eo:dbg]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#eo:dbg
