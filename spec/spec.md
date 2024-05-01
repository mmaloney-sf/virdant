# Virdant Spec

## Overview

Virdant is a strongly-typed hardware description language.

## Packages
A package is the unit of compilation for Virdant.
It consists of a single `.vir` file.

A package consists of a number of top-level declarations called **items**.
These include:

* module definitions
* type definitions

```
public module Foo {
    incoming clk : Clock;
    incoming in : Word[8];
    outgoing out : Word[8];

    reg buffer : Word[8] on clk;

    buffer <= in->add(1);
    out := buffer;
}
```

A module definition consists of a list of declarations.
Declarations are considered in-scope everywhere within the module.
Thus, the order of declarations is not significant.

**Component** declarations represent a simple piece of hardware that exists in a module.
A component has a name and a type.

Virdant has four **kinds** of components:

* `incoming`
* `outgoing`
* `wire`
* `reg`

The `incoming` and `outgoing` components represent ports in towards and out from the module respectively.
The `wire` component represents a named value, used for clarity or to create an alias to a complex expression.
The `reg` component represents a stateful value.

Note, we do not confuse the kind of a component with its type, since types.
The type indicates the type of data that flows through the component.

A **submodule** declaration nests an instance of one module inside another.

A submodule is declared with a name and a module definition.
Note, we do not confuse the module definition with its type.
Submodules are not expressions, and so they do not have a type.

A **target** is a component which we may connect to.

Inside of a module definition, each `outgoing`, `wire`, and `reg` component introduces a **local target**,
and each `incoming` component of a submodule, as defined by its module definition, introduces a **non-local target**.

A **connect** is an unnamed statement which associate the expression on the right hand side to the target on the left hand side.

There are two kinds of connections:

* `:=` (continuous)
* `<=` (latched)

Continuous connects, written `:=`, are always in effect.
This is used with `incoming`, `outgoing`, and `wire` components.

Latched connects, written `<=`, are only used for `reg` components, and take effect every clock cycle.

A module definition must supply exactly one connect statement for each target.

## Types

For any natural number `n`, `Word[n]` is an `n`-bit integer.
It is nominally unsigned.

For any type `T` and natural number `n`, `Vec[T, n]` is a vector of elements of type `T` with length `n`.

`Clock` is the type of clock signals.

## Expressions

### References

Any expression may reference any `incoming`, `wire`, `reg` of the defining module or any `outgoing` of a submodule.

For clarity, when referencing a `reg`, you read the current value (the value that was latched on the previous cycle)
rather than the value which is about to be latched.

### Literals

Constant values, such as `0`, `1`, `2`, etc. may be used as expressions.
Their bitwidth will be inferred whenever possible.
To give their bitwidth explicitly, use the notation `0w8` (read "0 with width 8"), etc.

You may also specify integers using binary or hexadecimal: `0b1011w4`, `0xffw8`, etc.
You may also use underscores to break up numbers however you like: `0b000_11`, etc.


### Vec Constructors

Vectors may be constructed with the syntax `[0, 1, 2]`.

### Type Ascription

For expressions which are well-typed, but whose type can't be inferred, you can use a type ascription: `x->as(Word[8])`.

### Method Calls

Types may have methods defined on them.
These vary according to the type.
