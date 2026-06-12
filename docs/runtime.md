# Runtime Model

**Value:** `src/value.rs`  
**Objects:** `src/object.rs`  
**Hash table:** `src/table.rs`

Defines the runtime representation of data: tagged values on the stack, heap-allocated objects, and the hash table used for globals and string interning.

## Value (Tagged Union)

```rust
#[derive(Clone, PartialEq)]
pub enum Value {
    NUMBER(f64),
    BOOL(bool),
    STR(GcRef<ObjString>),
    FUNCTION(GcRef<ObjFunction>),
    NIL,
}
```

Implements `Display` for printing (`print` statement, REPL output).

### Accessors

```rust
get_bool() -> Option<bool>
get_number() -> Option<f64>
get_string() -> Option<GcRef<ObjString>>
get_function() -> Option<GcRef<ObjFunction>>
```

### Equality and Truthiness

```rust
fn is_falsey(value: &Value) -> bool;
fn falsify(value: &Value) -> bool;      // alias for is_falsey
fn values_equal(v1: &Value, v2: &Value) -> bool;
```

`values_equal` requires same discriminant; cross-type comparison returns false. `FUNCTION` equality is **pointer equality** (`GcRef` PartialEq).

### Truthiness table

| Variant | Falsey when |
|---------|-------------|
| `NIL` | always |
| `NUMBER` | `== 0.0` |
| `BOOL` | `false` |
| `STR` | never |
| `FUNCTION` | never |

## Heap Objects

All heap types carry a `GcObject` header for GC linked-list and marking.

### ObjectType

```rust
pub enum ObjectType {
    STRING,
    FUNCTION,
    CLASS,  // reserved, unused
}
```

### ObjString

```rust
#[repr(C)]
pub struct ObjString {
    header: GcObject,
    pub s: String,
    pub hash: usize,  // FNV-1a style
}
```

- Created via `ObjString::from_string(s)`.
- Hash: offset basis `2166136261`, prime `16777619`.
- Interned through `Gc::intern` — deduplicated in GC's string table.

### ObjFunction

```rust
#[repr(C)]
pub struct ObjFunction {
    header: GcObject,
    pub arity: u8,
    pub chunk: Chunk,
    pub name: GcRef<ObjString>,
}
```

- `ObjFunction::new(name)` — empty chunk, arity 0.
- `Display`: empty name → `<script>`, else `<fn {name}>` (note: condition checks `name.s == ""` but prints script for non-empty — likely bug/inversion).

### ObjNativeFunction (stub)

```rust
pub struct ObjNativeFunction {
    header: GcObject,
    pub arity: u8,
    pub function: fn(&[Value]) -> Value,
}
```

Defined and implements `GcManaged`, but VM does not dispatch to native functions yet.

### GcManaged Trait

```rust
pub trait GcManaged {
    fn header(&self) -> &GcObject;
}
```

Required for `Gc::alloc<T: GcManaged>`.

## GcRef

```rust
pub struct GcRef<T> {
    pointer: NonNull<T>,
}
```

- `Copy + Clone + Deref + PartialEq + Eq`
- `GcRef::dangling()` for sentinel initialization
- Equality is pointer address equality

## Table (Hash Map)

Open-addressing hash table with tombstones. Keys are always `GcRef<ObjString>`; values are `Value`.

```rust
pub struct Table {
    count: usize,
    capacity: usize,       // power of 2
    entries: *mut Entry,   // raw allocated array
}

struct Entry {
    key: Option<GcRef<ObjString>>,
    value: Value,
}
```

### API

| Method | Purpose |
|--------|---------|
| `new()` | Empty table |
| `set(key, value) -> bool` | Insert/update; returns `true` if new key |
| `get(key) -> Option<Value>` | Lookup |
| `delete_entry(key) -> bool` | Tombstone delete |
| `find_string(s, hash) -> Option<GcRef<ObjString>>` | Intern table lookup by C string + hash |
| `iter() -> IterTable` | Iterate live entries |
| `add_all(&from)` | Copy all entries |

### Hashing / Probing

- Index: `hash & (capacity - 1)`
- Linear probing with wrap
- Max load factor: `0.75`
- Growth: min 8, else double capacity
- Tombstones: `key = None`, `value = BOOL(true)`

### Memory

Uses `std::alloc::{alloc, dealloc}` directly. `Drop` deallocates entry array.

## Where Tables Are Used

| Owner | Purpose |
|-------|---------|
| `Vm.globals` | Runtime global variables |
| `Gc.strings` | String intern pool (values are `NIL` placeholders) |

## GC Traversal from Runtime Types

When marking:

- `Value::STR` / `Value::FUNCTION` → mark object
- `ObjFunction` → mark `name`, mark each `chunk.constants` value
- `Table` iteration → mark keys and values

## Testing

- `value.rs` — getters, truthiness, equality
- `table.rs` — set/get, delete, iter, add_all, find_string