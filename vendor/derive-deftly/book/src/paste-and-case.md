<!-- @dd-navbar -->
<!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html">crate top-level</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html#overall-toc">overall toc, macros</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/"><strong>guide/tutorial</strong></a></em> ]</nav>

# Transforming names and strings

Often, it's useful to define new identifiers
based on existing ones,
or to convert identifiers into strings.

You _could_ use the existing [`paste`] crate for this,
or you can use a native facility provided by derive-deftly.

For example, suppose that you want
to define a template that makes a "discriminant" type
for your enumerations.
You want the new type to be named `FooDiscriminant`,
where `Foo` is the name of your existing type.
While you're at it, you want to add an `is_` function
to detect each variant.

You can do that like this:
```rust
# use derive_deftly::define_derive_deftly;
define_derive_deftly! {
    Discriminant:

    #[derive(Copy,Clone,Eq,PartialEq,Debug)]
    enum ${paste $tname Discriminant} {
        $(
            $vname,
        )
    }

    impl<$tgens> $ttype where $twheres {
       fn discriminant(&self) -> ${paste $tname Discriminant} {
          match self {
              $(
                  $vpat => ${paste $tname Discriminant}::$vname,
              )
          }
        }

        $(
            fn ${paste is_ ${snake_case $vname}} (&self) -> bool {
                self.discriminant() ==
                    ${paste $tname Discriminant} ::$vname
            }
        )
    }
}
```

Here we see a couple of new constructs.

First, we're using
[`${paste}`][x:paste]
to glue several identifiers together into one.
When we say `${paste $tname Discriminant}`,
we are generating a new identifier from `$tname`
(the type name)
and the word Discriminant.
So if the type name is `Foo`,
the new type will be called `FooDiscriminant`.

Second, we're using
[`${snake_case}`][case-changing]
to transform an identifier into `snake_case`
(that is, lowercase words separated by underscores).
We use this to turn the name of each variant (`$vname`)
into a name suitable for use in a function name.
So if a variant is called `ExampleVariant`,
`${snake_case $vname}`  will be `example_variant`,
and `${paste is_ ${snake_case $vname}}` will be
`is_example_variant`.

There are [other case-changers][case-changing]:
  * `${pascal_case my_ident}` becomes `MyIdent`.
    You can also write this as
    `${upper_camel_case ..}`.
  * `${lower_camel_case my_ident}` becomes `myIdent`.
  * `${shouty_snake_case MyIdent}` becomes `MY_IDENT`.
  * `${snake_case MyIdent}` becomes `my_ident`, as you've already seen.

You can abbreviate `${paste ...}` as `$<...>`.

<!--
  TODO: Should we show `$<...>` instead of `${paste ...}`?
  Maybe we should principally advertise `$<...>`, everywhere,
  and de-emphasise `${paste ...}`.
 -->

#### Pasting onto types

Here's a side-note:
You can use `${paste}` to append identifiers to a type
(like `$ttype` or `$ftype`),
not just an identifier.
When you do this, `${paste}` will
[do the right thing][identifier-pasting]
even if the type is generic.

For example, if `$ftype` is `Vec<u8>`,
then `${paste $ftype Builder}` will expand to
`VecBuilder<u8>`, not `Vec<u8>Builder`.

#### A note on syntax

In this last section,
you've seen a new syntax for the first time.
Both `${paste ident ident..}` and `${snake_case ident}`
are special cases of the following meta-syntax,
which derive-deftly uses everywhere:

[`${KEYWORD ARGS.. }`][arg-syntax]

In fact, if you want,
you can use this format
for all of the expansion macros you have already seen:
`$ttype` is just a shortened form for `${ttype}`,
`$fname` is just `${fname}`,
and so on.

Some keywords,
including some of those we've already seen,
can take named arguments.
The syntax for this is:

`${KEYWORD ARGNAME=VALUE ARGNAME=VALUE...}`

> For example, we can use this syntax to give optional arguments to `$vpat`;
> see the template syntax reference for more information.

If you ever need to write a literal `$`
(say, if you want to confuse yourself
by making derive-deftly-generated pattern macros)
you can write [`$$`][dollar-dollar].


[`paste`]: https://docs.rs/paste/latest/paste/
[dollar-dollar]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#dollar-dollar
[x:paste]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:paste
[case-changing]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#case-changing
[identifier-pasting]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:paste
[arg-syntax]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#named-and-positional-template-arguments-to-expansions-and-conditions


