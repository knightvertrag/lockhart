# Compiler Pipeline

**Facade:** `src/compiler.rs`  
**Parser:** `src/parser/`  
**Codegen:** `src/codegen/`  
**AST:** `src/ast/`

Lockhart compiles source in two phases: parse to AST, then lower AST to bytecode.

## Entry Point

```rust
pub fn compile(source: String, gc: &mut Gc) -> Result<GcRef<ObjFunction>, InterpretError> {
    let program = parser::parse(&source).map_err(|e| {
        InterpretError::InterpretCompileError(e.to_string())
    })?;
    codegen::compile_ast(&program, gc).map_err(|e| {
        InterpretError::InterpretCompileError(e.to_string())
    })
}
```

## Parser (`src/parser/`)

### Public API

```rust
pub fn parse(source: &str) -> Result<Program, ParseError>;
```

- **GC-free** — raw strings in AST literals; no bytecode emission
- **Expressions** — Pratt parser via `parse_rule.rs` + `precedence.rs`
- **Statements** — recursive descent in `mod.rs`
- **Errors** — `ParseError` with `[line N]` messages
- **Spans** — start line captured at construct begin via `span_since(start, line)`

### Statement → AST mapping

| Construct | AST output |
|-----------|------------|
| `let x = e;` | `Decl::Var` |
| `fn f(a) { ... }` | `Decl::Function` |
| `print e;` | `Stmt::Print` |
| `if (c) s [else s]` | `Stmt::If` |
| `while (c) s` | `Stmt::While` |
| `for (init; cond; inc) s` | `Stmt::For` |
| `return [e];` | `Stmt::Return` (error if outside function) |

## Codegen (`src/codegen/`)

### Public API

```rust
pub fn compile_ast(program: &Program, gc: &mut Gc) -> Result<GcRef<ObjFunction>, CompileError>;
```

`Codegen` in `emit.rs` walks the AST:

- Scope management (`begin_scope` / `end_scope`, locals table)
- Jump patching for `if`, `while`, `for`, `and`, `or`
- String interning and nested `ObjFunction` allocation via `Gc`
- Global/local variable resolution at compile time

### Planned refactor

Move name resolution to a semantic pass (see [ast-migration.md](./ast-migration.md) Phase 1). Refactor emit to implement `Visitor` (Phase 2).

## Testing

| Layer | File | Count |
|-------|------|-------|
| Parser | `src/parser/tests.rs` | 5 |
| AST pretty | `src/ast/pretty.rs` | 3 |
| VM integration | `src/vm/tests.rs` | 11 |
| Other units | lexer, chunk, value, table | 15 |

```bash
cargo test   # 34 total
```