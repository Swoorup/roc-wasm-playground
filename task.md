Console CLI via wasmtime.

Milestone #1:

* Roc compiler embedded, compiles to wasm.
* wasmtime interprets it.
* Result passed back to stdout

Milestone #2:

* Host function calling

Milestone #3:

* Roc function calling

Milestone #4:

* Remove repl_platform.c, uses Rust, calling using Rust struct, enum, roc glue?
