# Entry Points and Execution Modes

**Main:** `src/main.rs`  
**REPL:** `src/repl.rs`  
**File execution:** `src/source.rs`

## CLI (`main.rs`)

```rust
fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("===============Lockhart initiated===============");
        repl::start();
    } else {
        let code = open_source_file(&args[1]);
        execute(code);
    }
    Ok(())
}
```

| Invocation | Behavior |
|------------|----------|
| `cargo run` | Start REPL |
| `cargo run -- test.lh` | Execute file |

Module tree declared in `main.rs`:

```
bytecode, chunk, compiler, gc, lexer, object, repl, source, table, token, value, vm
```

## REPL (`repl.rs`)

Uses `rustyline::Editor` for line input.

```rust
pub fn start() {
    let mut rl = Editor::<()>::new();
    let mut interpreter = Vm::init_vm();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                interpreter.interpret(line).unwrap_or_else(|err| {
                    println!("{:?}", err);
                });
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => panic!("{}", err),
        }
    }
}
```

Characteristics:

- **Single persistent VM** — globals survive across lines.
- **No multi-line input** — each line is a complete compilation unit (must be valid declarations/statements ending appropriately).
- Errors printed via `Debug` format, execution continues.
- Ctrl-C / Ctrl-D exit the loop.

## File Execution (`source.rs`)

```rust
pub fn open_source_file(file_name: &str) -> String;
pub fn execute(code: String);
```

- `open_source_file` — reads entire file to `String`; panics on I/O error.
- `execute` — fresh `Vm::init_vm()`, runs `interpret`, prints `Error: {:?}` on failure.

Each file run gets a **new VM** (no persisted state).

## Execution Pipeline (shared)

Both modes ultimately call:

```
Vm::interpret(source)
  → compile(source, &mut vm.gc)     // compiler.rs
  → push script function
  → call(function, 0)
  → run()                           // opcode loop
```

## Example Program (`test.lh`)

```lh
fn loop(a, b) {
    return a + b;
}

print loop(10, 20);
```

Note: `loop` is a user-defined function name (not a keyword).

## Build and Test

```bash
cargo build
cargo test      # 24 tests
```

## VS Code

`.vscode/launch.json` and `settings.json` exist for IDE debugging (not part of runtime architecture).

## Implications for LLM-Assisted Development

- **REPL testing**: wrap snippets as complete statements with semicolons.
- **Global persistence**: only in REPL — re-run `Vm::init_vm()` in tests for isolation.
- **Error surfaces**: parser panics crash REPL line; VM errors are caught and printed.
- **No module system**: single compilation unit per `interpret` call.