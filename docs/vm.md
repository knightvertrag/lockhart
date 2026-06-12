# Virtual Machine

**Source:** `src/vm.rs`  
**Tests:** `src/vm/tests.rs`

Stack-based bytecode interpreter. One `Vm` instance owns the GC, operand stack, call frames, and global variable table.

Compilation reaches the VM through `compiler::compile` → `parser::parse` → `codegen::compile_ast`. See [compiler.md](./compiler.md).

## Vm State

```rust
pub struct Vm {
    gc: Gc,
    frames: [CallFrame; MAX_FRAMES],  // MAX_FRAMES = 64
    frame_count: usize,
    stack: Vec<Value>,                 // capacity MAX_STACK = 255
    stack_top: usize,
    globals: Table,
}
```

```rust
#[derive(Clone, Copy)]
struct CallFrame {
    function: GcRef<ObjFunction>,
    ip: *const (Opcode, Lineno),  // raw pointer into chunk.code
    slot: usize,                   // stack index of this frame's base slot
}
```

- **`slot`** — index in `stack` where this function's locals begin. Slot 0 of the frame is the function object for calls; parameters follow.
- **`ip`** — instruction pointer advanced each opcode. Jump/loop mutate via pointer arithmetic.

## Lifecycle

```rust
pub fn init_vm() -> Vm;
pub fn interpret(&mut self, source: String) -> Result<(), InterpretError>;
```

`interpret` compiles source, pushes the script function, calls it with 0 args, runs until `OP_RETURN` on the outermost frame.

## Stack Operations

```rust
fn push(&mut self, value: Value);      // stack[stack_top++] = value
fn pop(&mut self) -> Value;            // return stack[--stack_top]
fn peek(&self, idx: usize) -> &Value;  // stack[stack_top - 1 - idx]
```

Stack grows upward. `peek(0)` is TOS, `peek(1)` is second from top.

## Calling Convention

### Stack layout at call site

```
[..., callee, arg1, arg2, ..., argN]  ← stack_top
         ↑ slot = stack_top - 1 - argc
```

`OP_CALL(argc)` → `call_value(argc)`:

1. Peek callee at `peek(argc)`.
2. Must be `Value::FUNCTION`.
3. `call(func, argc)` checks arity, pushes new `CallFrame`:
   - `slot = stack_top - 1 - argc`
   - `ip = function.chunk.code.as_ptr()`

### Return

`OP_RETURN`:

1. Pop return value.
2. Decrement `frame_count`.
3. If `frame_count == 0`: pop script function, return `Ok(())`.
4. Else: reset `stack_top` to caller's slot, push return value, continue caller frame.

## Opcode Dispatch

`run()` is an infinite loop (until return) using `unsafe` pointer access to the active frame's `ip`:

```rust
let op = (*(*frame_ptr).ip).0;
(*frame_ptr).ip = (*frame_ptr).ip.offset(1);
match op { ... }
```

After `OP_CALL`, `frame_ptr` is refreshed to the top frame.

### Notable implementations

**`OP_ADD`** — dual semantics:
- Two strings → concatenate (order: `s2 + s1` pop order), intern result.
- Two numbers → add.
- Else → `InterpretRuntimeError`.

**`OP_JUMP_IF_FALSE`** — peeks TOS, jumps if `Value::is_falsey`. Does not pop.

**`OP_SET_GLOBAL`** — uses `Table::set` return value: `true` means new key → treated as undefined variable error (delete tombstone and error).

**Binary ops** — `binary_op!` macro pops two numbers, pushes result; panics on type mismatch (except `OP_ADD`).

## Truthiness (`Value::is_falsey`)

| Value | Falsey? |
|-------|---------|
| `NIL` | yes |
| `NUMBER(0)` | yes |
| `BOOL(false)` | yes |
| Everything else | no |

Used by `OP_NOT`, `OP_JUMP_IF_FALSE`, and logical `and`/`or` compilation.

## Error Types

```rust
pub enum InterpretError {
    InterpretCompileError(String),
    InterpretRuntimeError(String),
}
```

Runtime errors returned for: bad negate, bad add, undefined global, wrong arity, stack overflow (frames), calling non-function.

Many VM paths still `panic!` (binary op type mismatch, etc.).

## Garbage Collection Integration

Each opcode dispatch iteration:

```rust
if self.gc.should_collect() {
    self.collect_garbage();
}
```

```rust
fn collect_garbage(&mut self) {
    self.mark_roots();
    self.gc.collect_garbage();
}

fn mark_roots(&mut self) {
    // mark all stack values [0..stack_top)
    // mark all frame functions
    // mark globals table
}
```

## Globals

`Table` keyed by `GcRef<ObjString>`. Name constants in bytecode resolve to interned strings via `read_constant`.

## Integration Tests (`vm/tests.rs`)

Helper functions:

```rust
fn run(source: &str) -> Vm;
fn run_err(source: &str) -> InterpretError;
fn global(vm: &mut Vm, name: &str) -> Value;
```

Coverage: arithmetic/precedence, string concat, boolean logic, if/else, while, for, functions, block assignment, undefined variable, wrong arity, invalid operands.

## Performance / Safety Notes

- Raw instruction pointers avoid bounds checks in the hot loop.
- `CallFrame` is `Copy` — frames array stores by value.
- `MAX_STACK = 255` but stack is `Vec` — no overflow check on push.
- Frame pointer refresh after call is manual.

## Extension Points

- **`ObjNativeFunction`** in `object.rs` — not handled in `call_value` yet.
- **`OP_MOD`** — implemented but unused by compiler.
- Disassembly hook exists but is commented out in `run()`.