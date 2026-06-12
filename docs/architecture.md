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

## Entry Points

| Entry | File | Behavior |
|-------|------|----------|
| CLI (no args) | `src/main.rs` → `src/repl.rs` | Interactive REPL via `rustyline` |
| CLI (file arg) | `src/main.rs` → `src/source.rs` | Read `.lh` file and execute |
| Tests | `src/vm/tests.rs`, module `#[cfg(test)]` | End-to-end VM integration tests |

Both entry paths create a `Vm`, call `Vm::interpret(source)`, which compiles then runs.

## Module Map

| Module | Path | Responsibility |
|--------|------|----------------|
| Lexer | `src/lexer.rs` | Character stream → `Token` stream |
| Tokens | `src/token.rs` | `TokenType`, `Token`, keyword/operator maps |
| AST | `src/ast/` | Expression, statement, declaration nodes |
| Parser | `src/parser/` | Pratt expression parser + recursive-descent statements |
| Parse rules | `src/parser/parse_rule.rs` | Token → prefix/infix handler table |
| Precedence | `src/parser/precedence.rs` | Operator precedence levels |
| Codegen | `src/codegen/` | AST → bytecode emission |
| Compiler facade | `src/compiler.rs` | `parse` → `compile_ast` entry point |
| Bytecode | `src/bytecode.rs` | `Opcode` enum |
| Chunk | `src/chunk.rs` | Bytecode + constant pool per function |
| VM | `src/vm.rs` | Stack machine, call frames, opcode dispatch |
| Value | `src/value.rs` | Runtime tagged union (`NUMBER`, `BOOL`, `STR`, `FUNCTION`, `NIL`) |
| Object | `src/object.rs` | Heap objects (`ObjString`, `ObjFunction`, `ObjNativeFunction`) |
| GC | `src/gc.rs` | Allocation, string interning, mark-and-sweep |
| Table | `src/table.rs` | Open-addressing hash table (globals, intern table) |
| REPL | `src/repl.rs` | Read-eval-print loop |
| Source | `src/source.rs` | File I/O and execution wrapper |
| Disassembler | `src/chunk/disassemble.rs` | Debug bytecode printing (not wired into main path) |

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

3. **Unified GC heap**: All heap objects (`ObjString`, `ObjFunction`) are allocated through `Gc::alloc`. Strings are interned via `Gc::intern` using a dedicated `Table`.

4. **Globals via constant pool**: Global variable names are interned strings stored in the chunk constant pool. Opcodes like `OP_GET_GLOBAL(idx)` index into that pool.

5. **Locals via stack slots**: Local variables map to stack slots relative to each call frame's `slot` offset. Scope exit emits `OP_POP` for each local.

6. **Error handling**: Parser returns `ParseError` with line numbers; `compile()` maps to `InterpretError::InterpretCompileError`. VM returns `InterpretRuntimeError` for runtime failures.

## Data Flow: `Vm::interpret`

```rust
// src/vm.rs
pub fn interpret(&mut self, source: String) -> Result<(), InterpretError> {
    let function = compile(source, &mut self.gc)?;
    self.push(Value::FUNCTION(function));
    self.call(function, 0)?;
    self.run()
}
```

1. `compile()` calls `parser::parse(source)` → `Program` AST.
2. `codegen::compile_ast(&program, gc)` lowers AST to opcodes in a script `ObjFunction`.
3. VM pushes the function, sets up initial `CallFrame`, enters `run()`.

## Dependencies

- `phf` — compile-time perfect hash maps for keywords, operators, delimiters (`token.rs`)
- `rustyline` — REPL line editing (`repl.rs`)

## Known Limitations / In-Progress


- `GEQ`, `LEQ`, `NEQ` compile to pairs of opcodes (`OP_LT` + `OP_NOT`, etc.) rather than dedicated instructions.
- `ObjNativeFunction` and `ObjectType::CLASS` exist in `object.rs` but are not wired into the VM.
- `OP_MOD` exists in bytecode but has no compiler emission path.
- Disassembler exists but is commented out in the VM hot loop.
- Local variable shadowing detection panics; no proper error recovery.
- Modified files on `dev` branch (`gc.rs`, `value.rs`, `object.rs`) reflect active GC/runtime work.

## Component Documentation

| Document | Topics |
|----------|--------|
| [lexer.md](./lexer.md) | Tokenization, comments, string/number/identifier rules |
| [tokens.md](./tokens.md) | `TokenType`, keyword maps, token structure |
| [ast.md](./ast.md) | AST node types, spans, visitor |
| [compiler.md](./compiler.md) | Parser, codegen, compile pipeline |
| [bytecode.md](./bytecode.md) | `Opcode` variants, `Chunk` layout, disassembler |
| [vm.md](./vm.md) | Stack, frames, opcode implementations, calling convention |
| [runtime.md](./runtime.md) | `Value`, `ObjFunction`, `ObjString`, `Table` |
| [gc.md](./gc.md) | Allocation, interning, mark-and-sweep, root marking |
| [entry-points.md](./entry-points.md) | `main`, REPL, file execution |

## Testing

```bash
cargo test
```

30 tests: lexer, parser, chunk/value/table unit tests, and VM integration tests covering arithmetic, strings, control flow, functions, compile errors, and runtime errors.

## File Extension

Source files use the `.lh` extension (Lockhart).