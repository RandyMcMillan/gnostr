- [Introduction](intro.md)

- [Getting started](getting-started.md)
  - [Before you begin](before-you-begin.md)
  - [A first example: derive_deftly(HelloWorld)](basic-template.md)

- [Using templates from other crates and modules](modularity.md)
  - [Templates inside modules](templates-in-modules.md)
  - [Exporting templates](exporting-templates.md)
  - [If you're only deriving once...](adhoc.md)

- [Example 1: Writing your own Clone](clone-intro.md)
  - [Working with structs and fields](clone-fields.md)
    - [Tuple and unit structs](clone-tuple-unit.md)
  - [Making MyClone apply to enumerations](clone-enums.md)
  - [Making MyClone apply to generics](clone-generics.md)
  - [Making MyClone apply conditionally](clone-conditionally.md)

- [Some more advanced topics](advanced-intro.md)
  - [Transforming names and strings](paste-and-case.md)
  - [Naming conventions for derive-deftly expansions](expansions-naming.md)

- [Example 2: Defining a constructor function](constructor-intro.md)
  - [Limiting a template to one kind of type](constructor-limitations.md)
  - [Debugging templates](constructor-debugging.md)
  - [Working with visibility](constructor-visibility.md)
  - [Using attributes to make a template take arguments](constructor-attrs.md)
  - [Getting started if/then/else conditionals](constructor-conditionals.md)
  - [Making Constructor set fields with Default](constructor-advanced-conditionals.md)
  - [Managing complexity with $define](constructor-define.md)

- [Example 3: A type to track the differences between objects](difference-intro.md)
  - [Some difficulties when creating a Diff type](difference-problems.md)
  - [A brute-force approach for applying to structs and enums](difference-if-enum.md)
  - [A better solution to our Diff problems](difference-solving.md)

- [Other features](other-features.md)
