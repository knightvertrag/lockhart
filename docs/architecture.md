# Lockhart Architecture

Lockhart is a bytecode interpreter for a small dynamically-typed language, written in Rust. Source code (`.lh` files) is lexed into tokens, parsed into an AST, compiled to bytecode, and executed on a stack-based virtual machine with mark-and-sweep garbage collection.

This document is the entry point for LLM context. See the linked component docs for implementation detail.

## Pipeline Overview

```
Source (.lh)
    │
    ▼
┌─────────┐     ┌──────────┐     ┌──────────┐     ┌─────────┐
│  Lexer  │ ──► │  Parser  │ ──► │ Codegen  │ ──► │   VM    │
│ lexer.rs│     │ parser/  │     │ codegen/ │     │  vm.rs  │
│ token.rs│     │ ast/     │     │compiler  │     │         │
└─────────┘     └──────────┘     └──────────┘     └────┬────┘
                         │              │               │
                         ▼              ▼               ▼
                    ┌────────┐    ┌─────────┐    ┌──────────────┐
                    │  AST   │    │ Chunk   │    │ Gc + Table   │
                    │ ast/   │    │ chunk/  │    │ gc.rs        │
                    └────────┘    └─────────┘    └──────────────┘
```

Debug path (no VM): `lockhart --dump-ast file.lh` → parse only → tree/JSON output.

## Entry Points

| Entry | File | Behavior |
|-------|------|----------|
| CLI (no args) | `src/main.rs` → `src/repl.rs` | Interactive REPL via `rustyline` |
| CLI (file arg) | `src/main.rs` → `src/source.rs` | Read `.lh` file and execute |
| CLI (`--dump-ast`) | `src/main.rs` | Parse and pretty-print AST |
| Tests | `src/*/tests.rs`, `#[cfg(test)]` | Unit and integration tests |

Execution paths call `Vm::interpret(source)` which compiles then runs. AST dump calls `parser::parse` only.

## Module Map

| Module | Path | Responsibility |
|--------|------|----------------|
| Lexer | `src/lexer.rs` | Character stream → `Token` stream |
| Tokens | `src/token.rs` | `TokenType`, `Token`, keyword/operator maps |
| AST | `src/ast/` | Expression, statement, declaration nodes |
| AST pretty | `src/ast/pretty.rs` | Tree and JSON dump |
| Parser | `src/parser/` | Pratt expressions + recursive-descent statements |
| Parse rules | `src/parser/parse_rule.rs` | Token → prefix/infix handler table |
| Precedence | `src/parser/precedence.rs` | Operator precedence levels |
| Codegen | `src/codegen/` | AST → bytecode emission |
| Compiler facade | `src/compiler.rs` | `parse` → `compile_ast` entry point |
| Bytecode | `src/bytecode.rs` | `Opcode` enum |
| Chunk | `src/chunk.rs` | Bytecode + constant pool per function |
| VM | `src/vm.rs` | Stack machine, call frames, opcode dispatch |
| Value | `src/value.rs` | Runtime tagged union |
| Object | `src/object.rs` | Heap objects (`ObjString`, `ObjFunction`) |
| GC | `src/gc.rs` | Allocation, string interning, mark-and-sweep |
| Table | `src/table.rs` | Open-addressing hash table |
| REPL | `src/repl.rs` | Read-eval-print loop |
| Source | `src/source.rs` | File I/O and execution wrapper |
| Disassembler | `src/chunk/disassemble.rs` | Debug bytecode printing |

## Language Features (Implemented)

- Literals: numbers, strings, `true`/`false`/`nil`
- Arithmetic: `+ - * /` (`+` also concatenates strings)
- Comparisons: `> < >= <= == !=`
- Logical: `and`, `or`, `!`
- Variables: `let x = ...;`, assignment `x = ...;`
- Blocks and lexical scopes: `{ ... }`
- Control flow: `if/else`, `while`, `for`
- Functions: `fn name(a, b) { ... }`, calls, `return`
- Built-in: `print expr;`

## Key Design Decisions

1. **Parse-then-compile**: Source is parsed into an AST (`src/ast/`), then lowered to bytecode by `src/codegen/`. The Pratt parser builds expression trees; statements use recursive descent.

2. **Bytecode VM**: Stack-based execution with call frames. Each `ObjFunction` owns a `Chunk` (instruction list + constant pool).

3. **Unified GC heap**: All heap objects are allocated through `Gc::alloc`. Strings are interned via `Gc::intern`.

4. **Globals via constant pool**: Global variable names are interned strings in the chunk constant pool.

5. **Locals via stack slots**: Local variables map to stack slots relative to each call frame's `slot` offset.

6. **Error handling**: Parser returns `ParseError`; codegen returns `CompileError`; both map to `InterpretCompileError`. VM returns `InterpretRuntimeError`.

7. **Source locations**: Tokens carry `start`/`end` byte offsets and `lineno`. AST `Span` records start line; parser uses `span_since(start, line)`.

## Data Flow: `Vm::interpret`

```rust
pub fn interpret(&mut self, source: String) -> Result<(), InterpretError> {
    let function = compile(source, &mut self.gc)?;
    self.push(Value::FUNCTION(function));
    self.call(function, 0)?;
    self.run()
}
```

1. `compile()` calls `parser::parse(source)` → `Program` AST.
2. `codegen::compile_ast(&program, gc)` lowers AST to opcodes.
3. VM pushes the function, sets up `CallFrame`, enters `run()`.

## Dependencies

| Crate | Used by |
|-------|---------|
| `phf` | Keyword/operator/delimiter maps (`token.rs`) |
| `rustyline` | REPL line editing |
| `serde` / `serde_json` | AST JSON dump (`ast/pretty.rs`) |

## Known Limitations

- `GEQ`, `LEQ`, `NEQ` compile to opcode pairs (`OP_LT` + `OP_NOT`, etc.).
- `ObjNativeFunction` and `ObjectType::CLASS` exist but are not wired into VM.
- `OP_MOD` exists in bytecode but has no codegen emission path.
- Disassembler exists but is not wired into the main execution path.
- Codegen duplicate-local detection returns `CompileError`; some codegen paths still use `unwrap` on internal stmt emission.
- No semantic analysis pass yet — see [ast-migration.md](./ast-migration.md).

## Component Documentation

| Document | Topics |
|----------|--------|
| [lexer.md](./lexer.md) | Tokenization, comments, line numbers |
| [tokens.md](./tokens.md) | `TokenType`, keyword maps, spans |
| [ast.md](./ast.md) | AST nodes, dump tooling |
| [ast-migration.md](./ast-migration.md) | Future AST roadmap |
| [compiler.md](./compiler.md) | Parser, codegen, compile pipeline |
| [bytecode.md](./bytecode.md) | `Opcode` variants, `Chunk` layout |
| [vm.md](./vm.md) | Stack machine, opcode dispatch |
| [runtime.md](./runtime.md) | `Value`, objects, `Table` |
| [gc.md](./gc.md) | Mark-and-sweep, interning |
| [entry-points.md](./entry-points.md) | CLI, REPL, AST dump |

## Testing

```bash
cargo test   # 34 tests
```

Coverage: lexer (including multiline lineno), parser AST shape, AST pretty-print, chunk/value/table units, VM integration (arithmetic, control flow, functions, compile/runtime errors).

## File Extension

Source files use the `.lh` extension (Lockhart).