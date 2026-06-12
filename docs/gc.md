# Garbage Collector

**Source:** `src/gc.rs`  
**Related:** `src/object.rs`, `src/value.rs`, `src/table.rs`, `src/vm.rs`

Mark-and-sweep collector with a unified object list, string interning, and tri-color marking via an explicit grey stack.

## Gc State

```rust
pub struct Gc {
    bytes_allocated: usize,
    next_gc: usize,              // threshold; starts at 1 MiB
    first: Option<NonNull<GcObject>>,  // intrusive linked list of all objects
    strings: Table,              // intern table: ObjString → NIL
    grey_stack: Vec<NonNull<GcObject>>,
}
```

## Object Header

```rust
#[repr(C)]
pub struct GcObject {
    marked: Cell<bool>,
    next: Cell<Option<NonNull<GcObject>>>,
    obj_type: ObjectType,
    size: usize,
}
```

All heap objects embed `GcObject` as first field (`#[repr(C)]`) for uniform traversal.

## Allocation

```rust
pub fn alloc<T: GcManaged>(&mut self, object: T) -> GcRef<T>
```

1. Increment `bytes_allocated` by `size_of::<T>()`.
2. `Box::new(object)` → leak to raw pointer via `Box::into_raw`.
3. Link header into `first` intrusive list (prepend).
4. Return `GcRef { pointer }`.

## String Interning

```rust
pub fn intern(&mut self, s: String) -> GcRef<ObjString>
```

1. Build `ObjString::from_string(s)` (computes hash).
2. `strings.find_string(&o_string.s, hash)` — if hit, return existing ref.
3. Else `alloc(o_string)`, insert into `strings` with `Value::NIL`, return ref.

Intern table ensures string equality can use pointer comparison after interning.

## Collection Trigger

```rust
pub fn should_collect(&self) -> bool {
    self.bytes_allocated > self.next_gc
}
```

VM checks this every opcode in `run()`.

## Mark Phase

### Roots (VM responsibility — `Vm::mark_roots`)

- All values on operand stack
- `function` in each active call frame
- All entries in `globals` table

### Mark API

```rust
pub fn mark_object<T: GcManaged>(&mut self, reference: GcRef<T>);
pub fn mark_value(&mut self, value: &Value);
pub fn mark_table(&mut self, table: &Table);
```

`mark_object`: if not already marked, set `marked = true`, push to `grey_stack`.

### Trace

```rust
fn trace_references(&mut self) {
    while let Some(object) = self.grey_stack.pop() {
        self.blacken_object(object);
    }
}
```

`blacken_object` by type:

| Type | Action |
|------|--------|
| `STRING` | nothing (no outgoing refs) |
| `FUNCTION` | mark `name`, mark each chunk constant via `mark_value` |
| `CLASS` | nothing (unused) |

## Sweep Phase

```rust
fn sweep(&mut self)
```

Walk intrusive list from `first`:

- **Marked** → clear mark, keep alive
- **Unmarked** → unlink, subtract `size` from `bytes_allocated`, `free_object`

```rust
fn free_object(&mut self, object: NonNull<GcObject>) {
    match obj_type {
        STRING => drop(Box::from_raw(...ObjString...)),
        FUNCTION => drop(Box::from_raw(...ObjFunction...)),
        CLASS => no-op,
    }
}
```

## String Table Cleanup

After tracing, before sweep:

```rust
fn remove_white_strings(&mut self)
```

Removes intern table entries whose `ObjString` was not marked (unreachable strings).

## Threshold Adjustment

After collection:

```rust
self.next_gc = (self.bytes_allocated * HEAP_GROW_FACTOR).max(1024 * 1024);
```

`HEAP_GROW_FACTOR = 2`.

## Gc Drop

On `Gc` drop, walks entire object list and frees everything (program shutdown cleanup).

## GcRef Safety Model

- `GcRef<T>` is a bare `NonNull<T>` with `Deref` → `unsafe` dereference.
- No generational or moving GC — pointers remain stable until sweep frees them.
- Collection only runs at opcode boundaries when VM invokes it — safe point for mutator.

## Interaction Diagram

```
Vm::run (each opcode)
    │
    ├─ should_collect? ──yes──► mark_roots()
    │                              ├─ stack values
    │                              ├─ frame functions
    │                              └─ globals table
    │                           collect_garbage()
    │                              ├─ trace_references (grey → blacken)
    │                              ├─ remove_white_strings
    │                              └─ sweep unmarked objects
    │
    └─ dispatch opcode
```

## Active Development Notes

Files `gc.rs`, `value.rs`, `object.rs` show modifications on the `dev` branch — GC and runtime model are under active refinement. When editing:

- New `ObjectType` variants need `blacken_object` and `free_object` arms.
- New `Value` variants holding heap refs need `mark_value` handling.
- Chunk constants may hold nested functions — `FUNCTION` marking traverses their constant pools recursively via `blacken_object`.