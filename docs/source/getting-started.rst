Getting Started
===============

Tradition dictates that the first program that is written in any programming language should be Hello, World!
In that spirit, in hardware languages, we write a design which blinks an LED on and off.
So let's make an LED blink by writing the "hello world" of Virdant:

.. literalinclude:: examples/blink.vir
   :caption: blink.vir
   :language: virdant
   :linenos:

We see a module declaration, `mod Blink`.
Inside, we see two ports, one `implicit` port named `clock` and one `outgoing` port named `led`.
The first port, `clock` has type `Clock`, and so it gives the circuit its pulse.
It is an implicit clock, so it is automatically fed to all `reg`\s which need a clock.
The second port, `led`, will represent the state of our LED.
When it's `true`, the LED is on, and when it's `false`, the LED is off.

The next line declares a `reg` called `led_on` with type `Bit`.
A `reg` is a hardware register.
It is a memory cell in our design.
The next line after that is a driver statement.
It tells us that every clock cycle, the value of `led_on` becomes `len_on->not()`.
In other words, it flips from `true` to `false` or `false` to `true` on every clock cycle.

The last line of the module, `led := led_on` is another driver statement.
It drives `led` continuously to have the same value as the current value of `led_on`.
