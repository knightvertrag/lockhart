# AST

**Source:** `src/ast/`  
**Produced by:** `src/parser/`  
**Consumed by:** `src/codegen/`

Lockhart parses source into a typed AST before bytecode emission.

## Pipeline

```
Lexer → Parser → Program AST → Codegen → ObjFunction
```

## Module Layout

| File | Contents |
|------|----------|
| `span.rs` | `Span`, `Spanned<T>`, `Ident` |
| `expr.rs` | `Expr`, literals, operators |
| `stmt.rs` | `Stmt`, `BlockStmt`, control flow |
| `decl.rs` | `Decl`, `VarDecl`, `FnDecl` |
| `visit.rs` | `Visitor` trait for traversal |
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
- `Binary` — arithmetic and comparisons (including `>=`, `<=`, `!=` as distinct ops)
- `Logical` — `and`, `or` (short-circuit preserved in tree; codegen emits jumps)
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

Every node is wrapped in `Spanned<T>` with byte offsets and line number from the lexer. Used for `ParseError` messages and future tooling.

## Codegen

`src/codegen/emit.rs` walks the AST and emits opcodes. Scope resolution, string interning, and `ObjFunction` allocation happen only in codegen — the parser is GC-free.