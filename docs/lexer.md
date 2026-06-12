# Lexer

**Source:** `src/lexer.rs`  
**Tests:** `src/lexer/tests.rs`  
**Depends on:** `src/token.rs`

The lexer converts a source `String` into a stream of `Token` values with byte offsets and line numbers.

## Public API

```rust
pub struct Lexer {
    input: String,
    position: usize,
    read_position: usize,
    ch: u8,
    lineno: usize,
}

impl Lexer {
    pub fn new(input: String) -> Lexer;
    pub fn next_token(&mut self) -> Token;
}
```

## Line Number Tracking

Line increments happen **only** in `read_char` when consuming `\n`. `skip_whitespace` does not increment separately — this avoids double-counting newlines between tokens.

At the start of each `next_token()`:

```rust
let start = self.position;
let lineno = self.lineno;  // captured before reading token bytes
```

## Scanning Algorithm

Each `next_token()` call (in a loop for comment skip):

1. `skip_whitespace` — skip ASCII whitespace via `read_char`
2. EOF check
3. Dispatch: operators (with `//` comment skip), strings, delimiters, identifiers, numbers
4. Return token with `start`, `end`, `lineno`

## Token Fields

| Field | Meaning |
|-------|---------|
| `literal` | Token text |
| `lineno` | Start line (1-based) |
| `start` / `end` | Byte offsets in source |

## Multi-Character Operators

| First char | Peek `=` | Result |
|------------|----------|--------|
| `=` | yes/no | `EQ` / `ASSIGN` |
| `>` | yes/no | `GEQ` / `GT` |
| `<` | yes/no | `LEQ` / `LT` |
| `!` | yes/no | `NEQ` / `NOT` |

## Comments

`//` line comments: consume until newline, return `ILLEGAL` token (discarded by parser loop on next call).

## Integration with Parser

```rust
// parser/mod.rs
fn advance(&mut self) {
    self.previous = self.current.clone();
    self.current = self.lexer.next_token();
}
```

Parser builds `ast::Span` from token offsets and start line. See [tokens.md](./tokens.md).

## Design Notes

- `TokenType` order must match `RULES` array in `parser/parse_rule.rs`
- Numbers: integer digits only; parsed to `f64` in parser
- Strings: no escape sequences
- Illegal chars: `panic!` (lexer-level; parser uses `ParseError`)