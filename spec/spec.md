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
pub mod Foo {
    incoming clk : Clock;
    incoming in : Word[8];
    outgoing out : Word[8];

    reg buffer : Word[8] on clk;

    buffer <= in->add(1);
    out := buffer;
}
```

A module definition consists of a list of statements.
The order of statements is not significant.

The following statements declare a **component**.
A component represents a simple piece of hardware that exists in a module.

* `incoming`
* `outgoing`
* `node`
* `reg`
* `mod`

The `incoming` and `outgoing` components represent ports in towards and out from the module respectively.
The `node` component represents a named value, used for clarity or to create an alias to a complex expression.
The `reg` component represents a stateful value.
The `mod` component represents a submodule instance.

Of these, `incoming` `outgoing` `node` and `reg` are called **simple components**.
Each simple component has a **type**.
At simulation time, each one will have a signal associated with it.

A `mod` component is where one module is nested inside of another.
A submodule is declared with a name and a module definition.

In addition to declarations, there are **wire** statements, which supply a value to a simple component.
These are written as `TARGET := EXPR` or `TARGET <= EXPR`.

The left hand side of a wire statement is the **target**.

Inside of a module definition, each `outgoing`, `node`, and `reg` component introduces a **local target**,
and each `incoming` component of a submodule, as defined by its module definition, introduces a **non-local target**.

The right hand side of a wire statement is an expression.
This expression will determine the values the target receives at runtime.

There are two kinds of wires:

* `:=` (continuous)
* `<=` (latched)

Continuous wires are used to connect to `incoming` `outgoing` and `node` components.
They continuously supply a value to the component.
This means that during simulation, the value of the target changes as soon as the value of the expression changes.

Latched wires are used to connect to `reg` components.
They supply a new value to the component on each tick of that register's associated clock.
You can think of them as being a wire to the register's data pin ("D" pin) in the underlying hardware.
This is in contrast to when a `reg` is used as an expression, which evaluates to the the register's output pin ("Q" pin).

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

Submodules are not expressions, and so they do not have a type.

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

#### Arithmetic
* `a->add(b)` - Add `b` to `a`. Both must have the same bitwidth.
* `a->sub(b)` - Subtract `b` from `a`. Both must have the same bitwidth.
* `a->neg()` - Negation of `a`. Result is the same type as `a`.

### Comparison
* `a->eq(b)` - Compare `a` to `b`. Both must have the same bitwidth. Result is a `Word[1]`.
* `a->lt(b)` - Less than. Compare `a` to `b`. Both must have the same bitwidth. Result is a `Word[1]`.
* `a->lte(b)` - Less than or equal. Compare `a` to `b`. Both must have the same bitwidth. Result is a `Word[1]`.
* `a->gt(b)` - Greater than. Compare `a` to `b`. Both must have the same bitwidth. Result is a `Word[1]`.
* `a->gte(b)` - Greater than or equal. Compare `a` to `b`. Both must have the same bitwidth. Result is a `Word[1]`.

### Logic
* `a->and(b)` - Logical AND `a` and `b`. Both must have the same bitwidth. Result is the same bitwidth.
* `a->or(b)` - Logical OR `a` and `b`. Both must have the same bitwidth. Result is the same bitwidth.
* `a->not()` - Logical NOT `a`. Result is the same bitwidth.

### Shifts
* `a->sll(b)` - Shift left logical. The values `a` and `b` may have different bitwidths. Result is the same type as `a`.
* `a->srl(b)` - Shift left logical. The values `a` and `b` may have different bitwidths. Result is the same type as `a`.

### Dynamic Index
* `a->get(i)` - Indexes into `a` fetching the bit in position `i`. When `i` is 0, this is the least significant bit. The bitwidth of `a` must be a power of 2 and the bitwidth of `i` must be the same as that power.


### Concatenation

You concatenate words together using `cat(x, y)`.
The results of each operand must be inferrable.

### Indexing

You index into words with `x[0]`, `x[1]`, `x[2]`, etc.
The index must be a constant literal.
`x[0]` is the least significant bit.

### Slicing

You slice into words with `x[1..0]`, `x[2..0]`,  `x[2..1]`, etc.
The indexes must be a constant literals.
For words, the higher index goes on the left.

The upper index is non-inclusive.
For example, if `x : Word[8]`, then `x` is the same as `x[8..0]`.
