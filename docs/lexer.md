# Lexer

**Source:** `src/lexer.rs`  
**Tests:** `src/lexer/tests.rs`  
**Depends on:** `src/token.rs`

The lexer converts a source `String` into a stream of `Token` values. It is a hand-written scanner with single-character lookahead (`peek_ahead`).

## Public API

```rust
pub struct Lexer {
    input: String,
    position: usize,      // index of ch
    read_position: usize, // index of next char to read
    ch: u8,               // current char (0 = EOF)
    lineno: usize,
}

impl Lexer {
    pub fn new(input: String) -> Lexer;
    pub fn next_token(&mut self) -> Token;
}

impl Iterator for Lexer {
    type Item = Token;
    // yields tokens until EOF, then None
}
```

The compiler uses `Lexer` directly via `lexer.next_token()` inside `Parser::advance()`.

## Scanning Algorithm

Each `next_token()` call:

1. **`skip_whitespace`** — skips ASCII whitespace; increments `lineno` on `\n`.
2. **EOF check** — if `read_position > input.len()`, return `TokenType::EOF`.
3. **Dispatch on `ch`**:
   - **Operators** — lookup in `token::OPERATORS` phf map. Multi-char operators use `build_double` (peek ahead for `=`, etc.).
   - **String** — `"` starts `read_literal()` (no escape sequences).
   - **Delimiters** — lookup in `token::DELIMITERS`.
   - **Identifier** — `is_letter` (ASCII alpha + `_`), then `read_identifier`. Keyword check via `Token::check_keyword`.
   - **Number** — `is_number` (ASCII digit), `read_identifier` with digit predicate.
   - **Illegal** — `panic!("illegal identifier")` for unrecognized chars.

4. **`read_char`** — advance `position`/`read_position`, update `ch`.

## Token Literal Contents

| Token kind | `literal` field |
|------------|-----------------|
| `NUM` | Raw digit sequence (parsed to `f64` in compiler) |
| `STRING` | Characters between quotes (no quotes in literal) |
| `IDENT` | Identifier text |
| Keywords | Keyword text (`let`, `fn`, etc.) |
| Operators/delimiters | The operator/delimiter character(s) |

Every token carries `lineno` — the line where the token **started** (used by compiler for `Lineno` in chunk emission).

## Multi-Character Operators

Handled in the `ASSIGN`, `GT`, `LT`, `NOT` match arms:

| First char | Peek `=` | Result |
|------------|----------|--------|
| `=` | yes | `EQ` (`==`) |
| `=` | no | `ASSIGN` (`=`) |
| `>` | yes | `GEQ` (`>=`) |
| `<` | yes | `LEQ` (`<=`) |
| `!` | yes | `NEQ` (`!=`) |
| `!` | no | `NOT` (`!`) |

## Comments

`//` line comments: when `/` is seen and next char is `/`, consume until newline (comment is not emitted as a token).

## Identifiers and Keywords

Identifiers: `[a-zA-Z_][a-zA-Z0-9_]*` (ASCII only).

Keywords resolved at lex time via `KEYWORDS` map in `token.rs`: `let`, `fn`, `print`, `true`, `false`, `return`, `if`, `else`, `and`, `or`, `for`, `while`, `nil`.

Non-keyword identifiers become `TokenType::IDENT`.

## Numbers

Integer digit sequences only (no decimal point, no exponent). The compiler parses `literal` as `f64` via `.parse::<f64>().unwrap()`.

## Strings

Delimited by `"`. No escape sequence support — backslashes and quotes inside strings are not handled specially. Reading stops at the next `"`.

## Error Behavior

- Unrecognized characters: `panic!("illegal identifier")`
- No recovery or error tokens (`ILLEGAL` is defined but not produced in normal paths)

## Integration with Compiler

```rust
// compiler.rs — Parser::advance
fn advance(&mut self) {
    self.previous = self.current.clone();
    self.current = self.lexer.next_token();
}
```

`Parser::new` creates the lexer; `parser.advance()` is called once before the parse loop to prime `current`.

## Design Notes for LLMs

- Lexer is **not** responsible for keyword semantics — only classification.
- `TokenType` discriminant order must match `RULES` array indexing in `parse_rule.rs` (both use `as usize` on the enum).
- Line numbers may be incremented both in `read_char` (on `\n`) and `skip_whitespace` — be careful when debugging lineno issues.