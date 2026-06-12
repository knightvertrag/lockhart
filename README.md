# Lockhart

Lockhart is a bytecode interpreter for a small dynamically-typed language, written in Rust. Source is lexed, parsed into an AST, compiled to bytecode, and executed on a stack-based VM with mark-and-sweep GC.

## Current Status

Core language features are implemented with 34 tests covering lexer, parser, codegen, and VM integration. The compiler uses a three-phase pipeline: **parse → AST → codegen**.

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

```
src/
  ast/          # AST node types, spans, pretty-printer
  parser/       # Pratt expression parser + recursive-descent statements
  codegen/      # AST → bytecode lowering
  compiler.rs   # compile() facade (parse → codegen)
  lexer.rs      # tokenization
  vm.rs         # bytecode execution
  gc.rs         # mark/sweep GC + string interning
  table.rs      # hash table (globals, intern table)
  value.rs      # runtime value model
  object.rs     # heap objects (strings, functions)
docs/           # architecture and component documentation
```

## Build

```bash
cargo build
```

## Run

REPL:

```bash
cargo run
```

Execute a file:

```bash
cargo run -- path/to/file.lh
```

Dump AST (tree or JSON):

```bash
cargo run -- --dump-ast test.lh
cargo run -- --dump-ast --format json test.lh
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

## Documentation

Architecture overview and component docs live in [`docs/`](docs/):

- [architecture.md](docs/architecture.md) — pipeline, module map, design decisions
- [ast.md](docs/ast.md) — AST nodes, spans, dump tooling
- [ast-migration.md](docs/ast-migration.md) — future AST roadmap
- [compiler.md](docs/compiler.md) — parser and codegen
- Additional component docs: lexer, tokens, bytecode, vm, runtime, gc, entry-points

## Notes

- Parse errors return structured `ParseError` with line numbers.
- Codegen errors return `CompileError`; both map to `InterpretCompileError` at the VM boundary.
- See [docs/ast-migration.md](docs/ast-migration.md) for planned AST improvements.