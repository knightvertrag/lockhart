# Lockhart

Lockhart is a small tree-walk/bytecode-style language runtime written in Rust.
It includes a lexer, Pratt parser + compiler, VM, hash table, and mark/sweep GC.

## Current Status

This is an in-progress interpreter implementation with a growing test suite.
Core execution paths are implemented and tested.

## Implemented Language Features

- Numeric literals and arithmetic: `+`, `-`, `*`, `/`
- Comparisons and equality: `>`, `<`, `>=`, `<=`, `==`, `!=`
- Boolean and nil literals: `true`, `false`, `nil`
- Logical operators: `and`, `or`, `!`
- String literals and string concatenation with `+`
- Variable declarations and assignment: `let x = ...;`, `x = ...;`
- Blocks and lexical scopes: `{ ... }`
- Control flow: `if/else`, `while`, `for`
- Function declarations and function calls
- `return` in functions
- `print` statements

## Project Structure

- `src/lexer.rs`: tokenization
- `src/compiler.rs`: parsing + bytecode emission
- `src/vm.rs`: bytecode execution
- `src/gc.rs`: mark/sweep garbage collector + string interning
- `src/table.rs`: hash table used by globals/interned strings
- `src/value.rs`, `src/object.rs`: runtime value/object model
- `src/vm/tests.rs`: VM behavior tests

## Build

```bash
cargo build
```

## Run

Start REPL:

```bash
cargo run
```

Run a source file:

```bash
cargo run -- path/to/file.lh
```

Example:

```lh
fn add(a, b) {
  return a + b;
}

let x = add(2, 3);
print x;
```

## Test

```bash
cargo test
```

## Notes

- Runtime/compile errors are still evolving and may panic in some parser paths.
- The language and VM internals are under active development.
