# Tokens

**Source:** `src/token.rs`  
**Consumed by:** `src/lexer.rs`, `src/compiler.rs`, `src/compiler/parse_rule.rs`

Defines the token type system and compile-time lookup tables for keywords, operators, and delimiters.

## TokenType Enum

```rust
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum TokenType {
    IDENT, NUM, STRING,
    LET, FUNCTION, IF, ELSE, FOR, WHILE, PRINT, RETURN,
    TRUE, FALSE, NIL,
    ASSIGN, NOT, GT, LT, GEQ, LEQ, EQ, NEQ,
    PLUS, MINUS, MUL, DIV, AND, OR,
    COMMA, SEMICOLON, LBRACE, RBRACE, LPAREN, RPAREN,
    ILLEGAL, EOF,
}
```

**Important:** Variant declaration order determines discriminant values (`0..35`). The parse rule table `RULES` in `parse_rule.rs` is indexed by `token_type as usize` and has exactly **36** entries. Adding/reordering variants requires updating `RULES`.

## Token Struct

```rust
#[derive(Debug, Clone)]
pub struct Token {
    pub type_: TokenType,
    pub literal: String,
    pub lineno: usize,
}
```

- `PartialEq` / `Eq` compare `type_` and `literal`.
- `Hash` hashes only `literal` (used if tokens are hashed).
- `Token::new_def()` creates an `ILLEGAL` placeholder for parser initialization.

## Keyword Resolution

```rust
pub static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! { ... };

pub fn check_keyword(ident: &String) -> TokenType {
    KEYWORDS.get(ident).cloned().unwrap_or(TokenType::IDENT)
}
```

| Keyword | TokenType |
|---------|-----------|
| `let` | `LET` |
| `fn` | `FUNCTION` |
| `print` | `PRINT` |
| `true` | `TRUE` |
| `false` | `FALSE` |
| `return` | `RETURN` |
| `if` | `IF` |
| `else` | `ELSE` |
| `and` | `AND` |
| `or` | `OR` |
| `for` | `FOR` |
| `while` | `WHILE` |
| `nil` | `NIL` |

## Operator Map

```rust
pub static OPERATORS: phf::Map<&'static str, TokenType> = phf_map! { ... };
```

Single-character keys. Multi-char operators (`==`, `!=`, etc.) are built by the lexer, not stored in this map.

## Delimiter Map

```rust
pub static DELIMITERS: phf::Map<&'static str, TokenType> = phf_map! { ... };
```

Maps `{ } ( ) , ;` to their token types.

## Parser Token Utilities

The compiler's `Parser` uses these patterns:

```rust
fn check_token_type(&self, type_: TokenType) -> bool;
fn match_token(&mut self, type_: TokenType) -> bool;  // advance if match
fn consume(&mut self, type_: TokenType, err: &str);   // advance or panic
```

- `previous` — last consumed token (used for literal values, lineno).
- `current` — lookahead token.

## Grammar Hints (from token usage)

Statements and declarations are distinguished in `Parser::declaration()`:

- `fn` → function declaration
- `let` → variable declaration
- else → statement (`print`, block, `if`, `return`, `while`, `for`, or expression statement)

All statements/expressions end with `;` except block bodies and control-flow heads which use explicit `consume` calls for delimiters.