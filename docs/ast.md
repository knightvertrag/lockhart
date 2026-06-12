# AST

**Source:** `src/ast/`  
**Produced by:** `src/parser/`  
**Consumed by:** `src/codegen/`, `src/ast/pretty.rs`

Lockhart parses source into a typed AST before bytecode emission.

## Pipeline

```
Lexer → Parser → Program AST → Codegen → ObjFunction
                              └→ pretty (debug)
```

## Module Layout

| File | Contents |
|------|----------|
| `span.rs` | `Span`, `Spanned<T>`, `Ident` |
| `expr.rs` | `Expr`, literals, operators |
| `stmt.rs` | `Stmt`, `BlockStmt`, control flow |
| `decl.rs` | `Decl`, `VarDecl`, `FnDecl` |
| `visit.rs` | `Visitor` trait (not yet used by passes) |
| `pretty.rs` | Tree and JSON dump |
| `mod.rs` | `Program` root |

## Program Root

```rust
pub struct Program {
    pub declarations: Vec<Spanned<Decl>>,
}
```

## Expression Variants

- `Literal` — number, string, bool, nil
- `Variable` — identifier reference
- `Unary` — `-`, `!`
- `Binary` — arithmetic and comparisons (`>=`, `<=`, `!=` as distinct ops)
- `Logical` — `and`, `or` (short-circuit in codegen via jumps)
- `Assign` — `name = expr`
- `Call` — `callee(args...)`
- `Grouping` — parenthesized subexpression

## Statement Variants

- `Expression` — expr;
- `Print` — print expr;
- `Block` — `{ declarations... }`
- `If` — condition, then, optional else
- `While` — condition, body
- `For` — optional init/condition/increment, body (structured, not desugared)
- `Return` — optional value

## Declaration Variants

- `Var` — `let name [= init]`
- `Function` — `fn name(params) { body }`
- `Statement` — wraps any statement at block/top level

## Spans

Every node is wrapped in `Spanned<T>`:

```rust
pub struct Span {
    pub start: usize,  // byte offset
    pub end: usize,
    pub line: usize,   // start line (1-based)
}
```

Line numbers come from the lexer (`read_char` increments on `\n`). Parser captures start line at construct begin, not the closing token's line.

## Codegen

`src/codegen/emit.rs` walks the AST. Scope resolution, string interning, and `ObjFunction` allocation happen only in codegen.

## Dumping / Visualization

```bash
cargo run -- --dump-ast test.lh
cargo run -- --dump-ast --format json test.lh
```

| Format | Description |
|--------|-------------|
| Tree | Indented output with `[line N]` annotations |
| JSON | `"type"` field on every node; includes spans |

### VS Code launch configs

| Config | Behavior |
|--------|----------|
| Dump AST (current file) | Tree dump of `${file}` |
| Dump AST JSON (current file) | JSON dump of `${file}` |
| Dump AST (test.lh) | Tree dump of `test.lh` |

## Future Work

See [ast-migration.md](./ast-migration.md) for the roadmap: semantic pass, visitor refactor, error unification, and new language feature nodes.