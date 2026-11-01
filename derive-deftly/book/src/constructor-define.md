<!-- @dd-navbar -->
<!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html">crate top-level</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/index.html#overall-toc">overall toc, macros</a> | <a href="https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/"><strong>guide/tutorial</strong></a></em> ]</nav>

# Managing complexity with $define

Our constructor example has gotten fairly hard to read!
Before we take it any further, let's refactor it
to make it more clear what it's doing.

Here, we will use the `$define` and `$defcond` keywords
to define new aliases for some of our longer expansions and conditions.
This will make our intent more clear, and hopefully easier to read.

```rust
# use derive_deftly::define_derive_deftly;
define_derive_deftly! {
   Constructor:

   // First, we define some aliases.

   // (1)
   ${define FUNC_NAME {
        ${if tmeta(constructor(newfn)) {
            ${tmeta(constructor(newfn)) as ident}
        } else {
            new
        }}
    }}

   // (2)
   ${defcond F_DFLT fmeta(constructor(default))}

   // (3)
   ${define F_INITIALIZER
       ${if F_DFLT {
          ::std::default::Default::default()
       } else {
          $fpatname
       }}
   }

  // Now that's out of the way, here is the template expansion:
  impl<$tgens> $ttype where $twheres {
     $tvis fn $FUNC_NAME(
       $(
          ${when not(F_DFLT)}  $fpatname: $ftype ,
        )
     ) -> Self {
          Self {
              $( $fname: $F_INITIALIZER , )
          }
      }
   }
}
#
# use derive_deftly::Deftly;
# #[derive(Deftly)]
# #[derive_deftly(Constructor)]
# struct Foo {
#     #[deftly(constructor(default))]
#     s: Vec<String>,
#     n: u32,
# }
```

In the example above, we've introduced some aliases
for our trickier expressions,
so that our template is easier to read.

At `// (1)` and `// (3)`, we've used [`$define`][x:define]
to introduce new expansions.
The earlier one (`$FUNC_NAME`) is the name of our function;
the later one (`$F_INITIALIZER`) is the expression that we use
to initialize the current field.

At `// (2)`, we've used [`$defcond`][x:defcond]
to introduce a new condition, called `$F_DFLT`.
It is true whenever we have been told
to use the `Default` value for a the current field.

The `$defcond` and `$define` keywords
don't expand to anything themselves;
their only affect is to create the interpretation
of the keywords or conditions they define.

> NOTE: To avoid name conflicts,
> aliases created by `$define` and `$defcond`
> must be in `ALL_CAPS`.
>
> As a general practice, it's a good idea
> to prefix field-specific aliases with `F_`
> and variant-specific aliases with `V_`.

## How does the scoping work here?

You might have noted that, in our example above,
we define our new keywords and conditionals
at the start of our template.
In this context, there is no "current field"—
but two of our definitions use `fmeta`,
which requires the existence of a current field.
How does this work?

In brief:
the body of a definition is interpreted
_at the point where it is used_.
No matter where we put the `${define F_INITIALZER ...}`
in our template,
it will use the value of `${fmeta ...}`
from the point where we actually use `$F_INITIALIZER`.



[x:define]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:define
[x:defcond]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:define
