# Virdant

Virdant is a hardware description langauge.
It is a reimagination of the [FIRRTL](https://github.com/chipsalliance/firrtl-spec) hardware description language.
It is intended to be a human-readable intermediate target for a hardware generator, like [Chisel](https://github.com/chipsalliance/chisel).

# Specification
[The Virdant Specification](spec/spec.md)

## Rationale

Most programming languages go through one or two major iterations before settling on their stable form.
This is an attempt at taking all of the lessons that SiFive has learned about FIRRTL as a hardware description language and consolidating them into a robust design that will scale to meet the current and future needs of the company.

Virdant capitalizes on the strong points of FIRRTL.
It is a clean and elegant hardware description language, suitable both as a compiler target for Chisel as well as an aid to verification.

Virdant reworks several design points which have proven difficult to address through an incremental process to FIRRTL.
In doing so, we also clarify the technical role and business value of the solution occupying this important space in our stack.

## Name
Following the great Berkeley tradition of acronymic names, Virdant might stand for: Verbose Intermediate Representation of Designs And Netlist Transforms.

## Values
### Semantics
Virdant should have a clear and concise operational semantics.
This semantics should be formally documented.
We should abhor overly casual descriptions of what any construct means, or leave up to interpretation how any feature interacts with any other.

### Independence from Verilog
It is too often the case in our current approach, our designs reflect the quirks and idiosyncrasies of Verilog, since that is the most important backend target.
However, Virdant will be designed independently from Verilog.
This ensures that our decisions are made with deliberate intent and guards us against careless design.
Virdant shall ship with a reference implementation, which will help us prototype features before we commit to them.
Verilog will remain the primary emission target.

### Versioning
Virdant will have a proper versioning story. This includes not only having a prominent version number on the binary of each release, but also policies for stability guarantees and deprecation procedures. New versions should also be delivered with release notes which are comprehensible to its users. Documentation should be accurate and complete. Experimental features are released (or else discarded) in a timely manner.

### Human Readability
Virdant will retain FIRRTL's human-readability.
It will be suitable as a backend target for our Chisel generators.
We will place a greater emphasis on using Virdant as a tool for developer understanding of the design.
It will be suitable both as a debug aid for digital designers.
More importantly, it should be a central tool for digital verification to understand and verify our designs.

### Linkability
Virdant is designed from the ground up to be suitable for separately-compiled designs.
It includes a package system and the design encourages documentation of each module so that it may be quickly integrated into designs.

### Tooling
The simple design of Virdant makes it amenable to analysis.
It is intended not only as a syntax to perform the handoff between Chisel and MLIR.
Instead, it allows developers to answer questions about the design that are useful for digital design, verification, software, physical design, customer experience and other teams who need to understand and work with the design they have in their hands.
