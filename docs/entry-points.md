# Entry Points and Execution Modes

**Main:** `src/main.rs`  
**REPL:** `src/repl.rs`  
**File execution:** `src/source.rs`

## CLI (`main.rs`)

```rust
fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--dump-ast") {
        dump_ast_cli(&args);
        return Ok(());
    }
    if args.len() == 1 {
        repl::start();
    } else {
        execute(open_source_file(&args[1]));
    }
    Ok(())
}
```

| Invocation | Behavior |
|------------|----------|
| `cargo run` | Start REPL |
| `cargo run -- test.lh` | Execute file |
| `cargo run -- --dump-ast test.lh` | AST tree dump |
| `cargo run -- --dump-ast --format json test.lh` | AST JSON dump |

Module tree declared in `main.rs`:

```
ast, bytecode, chunk, codegen, compiler, gc, lexer, object,
parser, repl, source, table, token, value, vm
```

## AST Dump (`--dump-ast`)

Parse-only path — no VM, no GC:

```
open_source_file → parser::parse → ast::pretty::dump_program
```

- `--format tree` (default) — indented tree with line numbers
- `--format json` — structured JSON for external tools
- Exit code `1` on parse failure

See [ast.md](./ast.md) for VS Code launch configs.

## REPL (`repl.rs`)

Uses `rustyline::Editor` for line input.

- **Single persistent VM** — globals survive across lines
- **No multi-line input** — each line is a complete compilation unit
- Errors printed via `Debug` format; execution continues
- Ctrl-C / Ctrl-D exit

## File Execution (`source.rs`)

- `open_source_file` — reads file to `String`; panics on I/O error
- `execute` — fresh `Vm::init_vm()`, runs `interpret`, prints errors

Each file run gets a **new VM** (no persisted state).

## Execution Pipeline (REPL and file)

```
Vm::interpret(source)
  → compile(source, &mut gc)        // compiler.rs
      → parser::parse(source)       // AST
      → codegen::compile_ast(...)   // bytecode
  → push script function
  → call(function, 0)
  → run()
```

## VS Code Launch Configs

| Config | Purpose |
|--------|---------|
| Debug 'lockhart' repl | REPL in external terminal |
| Debug executable 'lockhart' | Run `test.lh` under debugger |
| Dump AST (current file) | Tree dump of active `.lh` file |
| Dump AST JSON (current file) | JSON dump of active file |
| Dump AST (test.lh) | Tree dump of `test.lh` |
| Debug unit tests | Run test binary under debugger |

Configs in `.vscode/launch.json`.

## Build and Test

```bash
cargo build
cargo test   # 34 tests
```

## Implications for LLM-Assisted Development

- **REPL testing**: wrap snippets as complete statements with semicolons
- **Global persistence**: only in REPL — use fresh `Vm` in tests for isolation
- **Parse errors**: structured `ParseError`, not panics
- **AST inspection**: use `--dump-ast` or VS Code dump configs before debugging codegen
- **No module system**: single compilation unit per `interpret` call