# Compiler Pipeline

**Facade:** `src/compiler.rs`  
**Parser:** `src/parser/`  
**Codegen:** `src/codegen/`  
**AST:** `src/ast/`

Lockhart compiles source in two phases: parse to AST, then lower AST to bytecode.

## Entry Point

```rust
pub fn compile(source: String, gc: &mut Gc) -> Result<GcRef<ObjFunction>, InterpretError> {
    let program = parser::parse(&source)?;
    codegen::compile_ast(&program, gc)
}
```

## Parser (`src/parser/`)

### Public API

```rust
pub fn parse(source: &str) -> Result<Program, ParseError>;
```

- **GC-free** — stores raw strings in AST literals; no bytecode emission
- **Expressions** — Pratt parser via `parse_rule.rs` + `precedence.rs`
- **Statements/declarations** — recursive descent in `mod.rs`
- **Errors** — `ParseError` with line numbers (replaces panics)

### Statement parsing

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

`Codegen` in `emit.rs` walks the AST and emits opcodes into `ObjFunction.chunk`:

- Scope management (`begin_scope` / `end_scope`, locals table)
- Jump patching for `if`, `while`, `for`, `and`, `or`
- String interning and nested function allocation via `Gc`
- Global/local variable resolution at compile time

### Opcode mapping

See [bytecode.md](./bytecode.md). Logical operators and control flow use the same jump patterns as the previous direct-to-bytecode compiler.

## Testing

- **Parser tests:** `src/parser/tests.rs` — AST shape and parse errors
- **Integration:** `src/vm/tests.rs` — end-to-end (29 tests)

```bash
cargo test
```