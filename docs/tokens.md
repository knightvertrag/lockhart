# Tokens

**Source:** `src/token.rs`  
**Consumed by:** `src/lexer.rs`, `src/parser/`

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

**Important:** Variant order determines discriminant values (`0..35`). The parse rule table `RULES` in `src/parser/parse_rule.rs` has exactly **36** entries indexed by `token_type as usize`.

## Token Struct

```rust
#[derive(Debug, Clone)]
pub struct Token {
    pub type_: TokenType,
    pub literal: String,
    pub lineno: usize,
    pub start: usize,
    pub end: usize,
}
```

- `lineno` — 1-based line where the token **starts**
- `start` / `end` — byte offsets into source for `Span` construction
- `Token::span()` → `ast::Span`
- `Token::new_def()` — `ILLEGAL` placeholder for parser initialization

## Keyword Resolution

| Keyword | TokenType |
|---------|-----------|
| `let` | `LET` |
| `fn` | `FUNCTION` |
| `print` | `PRINT` |
| `true` / `false` / `nil` | `TRUE` / `FALSE` / `NIL` |
| `return` | `RETURN` |
| `if` / `else` | `IF` / `ELSE` |
| `and` / `or` | `AND` / `OR` |
| `for` / `while` | `FOR` / `WHILE` |

Resolved at lex time via `KEYWORDS` phf map. Non-keywords become `IDENT`.

## Parser Token Utilities

```rust
fn check(&self, type_: TokenType) -> bool;
fn match_token(&mut self, type_: TokenType) -> bool;
fn consume(&mut self, type_: TokenType, expected: &str) -> Result<(), ParseError>;
```

- `previous` — last consumed token
- `current` — lookahead token

## Grammar Hints

`Parser::declaration()` dispatch:

- `fn` → function declaration
- `let` → variable declaration
- else → statement

All statements/expressions end with `;` except block bodies and control-flow heads.