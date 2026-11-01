# **Notes and plans (`NOTES.md`)**

# Future template features

## Stringification

See
[#64](https://gitlab.torproject.org/Diziet/rust-derive-deftly/-/issues/64).

This is a new expansion context, admitting:

 * Literal strings (including raw strings, but not byte strings)
 * Expansions (inner template code must also be valid in string context)
 * Doubled dollars `$$`

Not permitted are other kinds of non-expansion tokens,
including non-string literals, punctuation, or identifiers.

Expansions that are valid in token context are all allowed.
Leaf expansions expand to their string representation
(`TokenStream as Display`).
Expansions that yield types
are stringified *as types* (which has an impact on spacing).

The precise formatting of the output is not to be relied on.

### Keyword `${string ...}`

Body is a template valid in string context.

When expanded in token context, expands to a string literal,
whose value is all the content strings, concatenated.

Within `${string ...}`,
literal strings and
the results of inner `${string }` expansions (and `$"..."` templates)
are not re-quoted.

### Case conversion 

Case conversion is allowed inside stringification;
the contents must be a valid stringification content.
Case conversion is then applied to the stringified text.
So inside `${stringify }`,
case changing doesn't force pasting context.

Identifier pasing `${paste }` and `$< ... >` is also allowed,
and expands to the string representation of the type.

### Interaction with `$define`

User-defined expansions whose body is precisely `${string }` or `$"..."`
are not requoted in string context.
For other user defined expansions, the content is evaluated as tokens,
and that token stream is converted to a string
(requoting any string literals it may contain).

This is in semantics similar to `${paste }`,
and ensures that
any particular template text is statically
either string context, or not.

Examples:

```rust,ignore
${define A { ${string "a"} }}   $A
${define B { ${string "b"} }}   ${string $B}
${define C {          "c"  }}   ${string $C}
```

Expands to:

```rust,ignore
"a"
"b"
r#""c""#
```

### String templating `$"..."`

This is a convenient syntax for specifying a string template.
The leading `$` is followed by a (possbily raw) string literal.

The string literal contents is lexed as follows:

 * Literal text
 * Expansions in the form `$keyword` (or user-defined `$NAME`).
 * Doubled dollars `$$`
 * Possibly, at some point `${keyword ARGS...}` but with limitations.
    Possible limitation producing an initial level of support is:
     * the characters `"'` cannot appear in the `ARGS...`
 * Possibly, at some point `$< ... >` with the same limitation.

This template is equivalent to a `${string ...}`
containing the same elements.

Possibly in the future we might support
`${"template with $1 positional arguments" $< blarky $fname >}`
or similar.

### Doc attributes

We could support `$///` and `$//!`.
The compiler turns these into `#[doc(...)]` and `#![doc(...)]`.

So we would have to recognise `$#[doc` and `$#![doc`.
(This means we would find it hard (or maybe impossible)
to use `$#` for anything other than attributes.)

The literal would be processed as for `$"..."`.

### Byte strings

We could support byte literals maybe.
`${string }` doesn't have a way to request them;
there would probably have to be `${byte_string }`.
`$"..."` does have a natural way: `$b"..."`.

Supporting byte strings would mean allowing `b"..."`
within (the relevant) string context,
posing the possibility of
`${string b"non-utf8 bytes"}` which has to be an error.

When is this error checked?
Is there one kind of string context, or two?
Do we allow `${string b"valid utf8"}`?

We could detect the invalid utf-8 at expansion time.
But perhaps we should declare
that we reserve the right to check this statically,
and that wouldn't be a breaking change?

## Scopes (for meta, and loops)

See [discussion in #36](https://gitlab.torproject.org/Diziet/rust-derive-deftly/-/issues/36#note_3015721)

### Demo - cross product, allowing access to outer repetition

```rust,ignore
 ${for let a = fields {
   ${for let b = fields {
     self_cross_product_contains(${a.fname} ${b.fname});
   }}
 }}
```

### Fixing awkwardness re optional meta attributes

[#40](https://gitlab.torproject.org/Diziet/rust-derive-deftly/-/issues/40)

```rust,ignore
 ${if let m = fmeta(call) {
   ${m.meta as expr} (&self.$fname);
 }}
```

### Handling meta attribute multiplicity

[#36](https://gitlab.torproject.org/Diziet/rust-derive-deftly/-/issues/36)

```rust,ignore
 ${for let m = fmeta(call) {
   ${m.meta as expr} (&self.$fname);
 }}
```

### Looking for a thing in various places:

```rust,ignore
 ${if let m = any(fmeta(call), vmeta(fields(call)), tmeta(fields(call))) {..}}
 ${for let m = all(fmeta(call), vmeta(fields(call)), tmeta(fields(call))) {..}}
 ${if let m = select1(fmeta(call), fmeta(obsolete_name_for_call)) {..}}
```

### Meta scope referring to nonexistent meta item?

Can a meta item scope refer to a putative,
but actually nonexistent, item?
Not sure if we need this.
[#62](https://gitlab.torproject.org/Diziet/rust-derive-deftly/-/issues/62)
I suggest not, in the first instance.

```rust,ignore
 ${let m = fmeta(call) {
   ${if ${m.meta} { ... }}
 }}
```

### Details of binding in `if ... any` etc.

What about mixed binding and non-binding conditions?
Options are
(a) reject this
(b) on the non-binding arms, bind to nonexistent meta.
I think I prefer (a).
It's the more cautious approach, certainly.

```rust,ignore
 ${if let m = any(fmeta(call), true) {
   ${if ${m.meta} { ... }}
}}
```

### Meta scopes vs repeat scopes

Every scope is either a meta scope or a repeat scope.
`${scope.meta}` is only allowed for meta scopes.
Other expansions, including notably `${scope.Xmeta}`,
are only allowed for repeat scopes.

```rust,ignore
 ${for let m = fmeta(call) {
   .. ${m.fmeta(call)} .. // ERROR, user wrote `fmeta`, wanted `meta`
   .. ${m.meta(call)} .. // OK
 }}
 ${for let m = fields {
   .. ${m.fmeta(call)} .. // OK
   .. ${m.meta(call)} .. // ERROR, m isn't a meta scope
                         // user must write ${m.fmeta}
 }}
```

#### Alternative

With a meta scope `m`,
the only legal expansion using it is `${m.meta((call)}`
or whatever.

(Right now;
But expansions other than `$Xmeta` might be OK to support.
Not `$Xmeta` because it has too much scope for error:
since `${if let m = fmeta(call) { .. ${m.fmeta(..)} .. }}`
doesn't do at all what the user might expect.)

So, instead, we could say that you don't need to write `.meta`
and have it be just `${m(call)}`.  But:

 * Now it lives in the same namespace as keywords,
   and is a user-defined name, so it must be uppercase.
   `${M(call)}`.
 * This reduces the similarity between meta scopes and normal scopes.
 * It would prevent us supporting eg `${m.fname}` in the future,
   which might be useful with something like
   `${if let m = find1(field, fmeta(call)) { $m.fname ... }}`.
   (meaning "find the precisely one field with `#[deftly(call)]`,
   and then expand stuff with it),
   or other things I haven't thought of yet.
 * If we support arguments to user-defined meta items,
   the syntax for passing them wouldn't look like the meta syntax,
   so `${M(call)}` is syntactically weird.

### Binding and checking

Binding is dynamic (like `${define }`)
(despite the use of `let` which is often lexical
in other languages including Rust).

(Meta attribute checking is dynamic and precise, naturally.)

## Splitting off fields and handling subsets of the generics

Syntax and semantics TBD.  Some notes:

```text
   For things that need to split off fields        struct Foo as above {
   and talk only about subsets of the generics         field: Box<T>,
      generic parameter uses (for fields)
      		$fgens					T,
		$fgens_omitted				'l, C
      For explicit iteration, within ${for tgens ...} or $( ... )
		$tgname					'l   	T   	C
		$tgbounds ???

   Something for being able to handle structs/unions/enums
   equally in template, whatever that means.  We need to expand
   something to struct/union/enum, and possibly the brackets and
   commas in enum { ..., ..., }, and so on.
```

# Future plans wrt macro namespace questions

## Deriving from things other than data structures

It would be nice to be able to eventually support deriving from
items (traits, fns, ...).  This would have to be an attribute proc macro.  Attribute proc macros get to modify their applicand, but we would not do that.

Ideally that attribute macro would be `#[derive_deftly]`.  However:

 * We are already using that as an inert helper attribute for `#[derive(Deftly)]`.  Possibly we could experiment to see how that would interact with a non-inert attribute macro, except that:

 * It is not possible to have an attribute macro and a function-like macro with the same name; even though the invocation syntaxes (and implementing macro function signatures) are different.

## Proposed taxonomy of macros and attributes

We won't implement all of this right away,
but it is good to have a plan to make sure the names we take now
won't get in the way.

 * **`#[derive(Deftly)]`**:
   invokes the from-struct derivation machinery; enables:
    1. use of `#[derive_deftly(ReuseableMacro)]` on this very struct
    2. later use of `derive_deftly_adhoc!` of the same struct
	   (if `#[derive_defly_adhoc]` specified)
    3. `#[deftly(...)]` attributes on bits of the data structure
	   (checked via chaining technique).

 * **`define_derive_deftly!{ [export] MACNAME: TEMPLATE }`**:
   define a reusable template, which may be invoked as
   `#[derive_deftly(MACNAME)]`
   (within a struct annotated with `#[derive(Deftly)]` or
   `#[item_derive_deftly(MACNAME)]`.

 * **`derive_deftly_adhoc!{ DRIVERNAME: TEMPLATE }`**:
   adhoc derivation from something previously annotated with
   `#[derive(Deftly)]` or `#[item_derive_deftly]`.
   `DRIVERNAME` is an item path; we conflate the type and value namespaces.

 * **`#[derive_defly_adhoc]`**:
   Inert helper attribute to enable use of `derive_deftly_adhoc!`.

 * **`#[item_derive_deftly(MACNAME)]`**:
   attribute macro to be applied to items.
   The item is reproduced unchanged, except that
   `#[deftly]` attributes *in places where we would look for them*
   are filtered out.
   `#[item_derive_deftly]` will look forward to see if there are
   further `#[item_derive_deftly]` attributes,
   so that they can be combined and processed together
   (this is necessary for correctness of meta attr handling).
   Template *must* use `for ...` option.
   `#[derive_deftly_adhoc]` works as usual.
   It's an error to have `#[item_derive_deftly]` without the `()`.

 * **`#[deftly]`**:
   Inert helper attribute for `#[derive(Deftly)]`.
   Filtered-out attribute for `#[item_derive_deftly]`.
   Contents available via `$Xmeta`.

 * **`#[only_derive_deftly]`**:
   attribute macro to be applied to items;
   like `#[item_derive_deftly]` but *consumes and does not emit* the item.
   (We don't really need to be sure about this name;
   this will be an unusual case and we can give it whatever name seems good,
   later.)

## consume and not emit:

### Composition problem with `#[deftly]` attributes

You should be able to compose mutually-ignorant derive-deftly templates.
In particular you should be able to chain transforming templates,
and transforming templates should be able to
output invocations of normal deriving templates.

This won't work without doing "something",
because the outer invocation (the first to process the input)
will see a bunch of unrecognised `#[deftly]` attributes.

I'm not sure what the answer is,
but perhaps a template option for accepting `#[deftly]` attrs
and `${attr}` for transforming them.

Then the caller could `#[deftly(inner(foo))]`
and the inner template would receive `#[deftly(foo)]`.

Perhaps.

### Possible alternative syntax/naming

 * **`#[transform_deftly]`**:
   attribute macro to be applied to items.

 * d-d option `transform`.
   Insists that this template is for `#[transform_deftly]` only.

## `for ...` d-d options

Currently, we have `for enum`, `for struct`, `for union`.
These are fine.

We want also want ways to say:

 * `struct` or `enum`, not `union`: `for data`?
 * Particular kind of item: `fn`, `trait`, `mod`, `const`.
 * Any item: `item` (with `#[item_derive_adhoc]` for non-data items).
 * Combinations of the above: eg `for fn/const`?

Outstanding questions, therefore:

 * Does `for any` mean anything?
 * What keyword is "`struct`/`enum`"?
 * Do we need a keyword for `struct`/`enum`/`union`? 
   Probably, since this is going to be the default!
 * Is `/` the right separator for "or"?

### Internals

 * **`derive_deftly_engine!`**: Proc macro that does all the work.

 * **`derive_deftly_driver_DRIVERNAME!`**:
   `macro_rules` macro generated by `#[derive(Deftly)]` and
   `#[item_derive_deftly]`, embodying a driver.

 * **`derive_deftly_template_MACNAME!`**:
   `macro_rules` macro generated by `define_derive_deftly!`,
   embodying a template.

# Things to check before declaring 1.0

None!

But we should get some experience with the renamed crate,
probably by upgrading arti to it.
