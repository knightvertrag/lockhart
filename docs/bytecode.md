# Bytecode and Chunks

**Opcode definitions:** `src/bytecode.rs`  
**Chunk storage:** `src/chunk.rs`  
**Disassembler:** `src/chunk/disassemble.rs`

Each compiled function (including the top-level script) owns a `Chunk`: a sequence of `(Opcode, Lineno)` pairs plus a constant pool of `Value`s.

## Chunk Structure

```rust
#[derive(Clone, Copy)]
pub struct Lineno(pub usize);

pub struct Chunk {
    pub code: Vec<(Opcode, Lineno)>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Chunk;
    pub fn add_constant(&mut self, value: Value) -> usize;  // returns index
    pub fn write_chunk(&mut self, op: Opcode, lno: Lineno);
}
```

- **`code`** — instruction stream. Each instruction carries source line metadata for debugging.
- **`constants`** — literal pool: numbers, interned strings, nested `ObjFunction` values.

Constant indices are embedded in opcodes like `OP_CONSTANT(3)`.

## Opcode Reference

```rust
#[derive(Debug, Clone, Copy)]
pub enum Opcode {
    OP_CONSTANT(usize),
    OP_RETURN,
    // Arithmetic
    OP_NEGATE, OP_ADD, OP_SUBSTRACT, OP_MULTIPLY, OP_DIVIDE, OP_MOD,
    // Literals
    OP_TRUE, OP_FALSE, OP_NOT, OP_NIL,
    // Comparison
    OP_EQ, OP_GT, OP_LT,
    // Variables
    OP_DEFINE_GLOBAL(usize),
    OP_GET_GLOBAL(usize),
    OP_SET_GLOBAL(usize),
    OP_GET_LOCAL(usize),
    OP_SET_LOCAL(usize),
    // Stack / IO
    OP_PRINT, OP_POP,
    // Control flow
    OP_JUMP(usize),
    OP_JUMP_IF_FALSE(usize),
    OP_LOOP(usize),
    OP_CALL(u8),
}
```

### Opcode Semantics (VM behavior summary)

| Opcode | Stack effect | Notes |
|--------|--------------|-------|
| `OP_CONSTANT(i)` | push constants[i] | |
| `OP_NIL/TRUE/FALSE` | push literal | |
| `OP_NEGATE` | pop num, push -num | Runtime error if not number |
| `OP_ADD` | pop 2, push sum or concat | Strings or numbers |
| `OP_SUBSTRACT/MULTIPLY/DIVIDE/MOD` | pop 2 nums, push result | Panics on type mismatch |
| `OP_NOT` | pop, push bool | Via `Value::falsify` |
| `OP_EQ` | pop 2, push bool | `Value::values_equal` |
| `OP_GT/LT` | pop 2 nums, push bool | |
| `OP_DEFINE_GLOBAL(i)` | pop, store in globals | Name = constants[i] as string |
| `OP_GET_GLOBAL(i)` | push from globals | Error if undefined |
| `OP_SET_GLOBAL(i)` | peek assign to globals | Error if new key |
| `OP_GET_LOCAL(i)` | push stack[slot+i] | Relative to frame slot |
| `OP_SET_LOCAL(i)` | stack[slot+i] = peek | |
| `OP_PRINT` | pop, println | |
| `OP_POP` | pop | Used for expr-stmt, scope cleanup |
| `OP_JUMP(n)` | ip += n | Forward jump |
| `OP_JUMP_IF_FALSE(n)` | if peek falsey, ip += n | Does not pop condition |
| `OP_LOOP(n)` | ip -= n | Backward branch |
| `OP_CALL(argc)` | new call frame | Callee below args on stack |
| `OP_RETURN` | pop, unwind frame | Script return exits run loop |

### Jump Offset Encoding

Offsets are **relative to the instruction after the jump opcode**. The compiler computes:

- Forward: `chunk.len() - offset - 1`
- Loop: `chunk.len() - loop_start + 1` (backward)

## Constant Pool Usage

| Compiler action | Constant pool entry |
|-----------------|---------------------|
| Number literal | `Value::NUMBER(f64)` |
| String literal | `Value::STR(interned)` |
| Function value | `Value::FUNCTION(GcRef<ObjFunction>)` |
| Global/local name (global ops) | `Value::STR(interned identifier)` |

Local variable ops use **stack slot indices** in the opcode, not constant pool names.

## ObjFunction ↔ Chunk

```rust
// object.rs
pub struct ObjFunction {
    header: GcObject,
    pub arity: u8,
    pub chunk: Chunk,
    pub name: GcRef<ObjString>,
}
```

Every callable — script or `fn` — is an `ObjFunction` with its own `Chunk`.

## Disassembler

`src/chunk/disassemble.rs` provides debug output:

```rust
pub fn disassemble_chunk(chunk: &Chunk, name: &str);
pub fn disassemble_instruction(chunk: &Chunk, offset: usize);
```

Prints offset, lineno, and opcode. Constant-bearing instructions also print the constant value. Not enabled in production `Vm::run` (commented out).

## Unimplemented / Dead Opcodes

- **`OP_MOD`** — defined and implemented in VM, but compiler never emits it.
- No dedicated opcodes for `>=`, `<=`, `!=` — compiler emits opcode pairs.

## Testing

`chunk.rs` has unit tests for `add_constant` and `write_chunk`.

## LLM Modification Checklist

Adding a new opcode requires changes in **four places**:

1. `bytecode.rs` — enum variant
2. `compiler.rs` — emission site(s)
3. `vm.rs` — `run()` match arm
4. `chunk/disassemble.rs` — optional debug printing